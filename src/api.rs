//! A small client for the Iconify API (https://iconify.design/docs/api/).
//!
//! Only the read-only queries the CLI needs are covered: `/collections`,
//! `/search`, `/{prefix}.json?icons=` and `/{prefix}/{name}.svg`. Responses are
//! size-limited and parsed into typed views; unknown fields are ignored.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::id::IconId;

pub const DEFAULT_API: &str = "https://api.iconify.design";

/// The search endpoint accepts limits between these values.
pub const SEARCH_MIN_LIMIT: usize = 32;
pub const SEARCH_MAX_LIMIT: usize = 999;

const JSON_LIMIT: u64 = 8 * 1024 * 1024;
const SVG_LIMIT: u64 = 2 * 1024 * 1024;
/// Icon names per `/{prefix}.json` request, to keep URLs short.
const NAMES_PER_REQUEST: usize = 60;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Author {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct License {
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spdx: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// `IconifyInfo`: what the API says about an icon set.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SetInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(default)]
    pub author: Author,
    #[serde(default)]
    pub license: License,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// True when icons carry their own colors, false for `currentColor` icons.
    #[serde(default)]
    pub palette: bool,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub icons: Vec<String>,
    #[serde(default)]
    pub collections: BTreeMap<String, SetInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub limit: usize,
    pub start: usize,
    /// Restricts results to these icon sets; empty means all.
    pub prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IconData {
    pub body: String,
    pub left: Option<f64>,
    pub top: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotate: Option<i32>,
    #[serde(rename = "hFlip")]
    pub h_flip: Option<bool>,
    #[serde(rename = "vFlip")]
    pub v_flip: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AliasData {
    pub parent: String,
    pub left: Option<f64>,
    pub top: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rotate: Option<i32>,
    #[serde(rename = "hFlip")]
    pub h_flip: Option<bool>,
    #[serde(rename = "vFlip")]
    pub v_flip: Option<bool>,
}

/// `IconifyJSON` as returned by `/{prefix}.json?icons=`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IconSetData {
    #[serde(default)]
    pub icons: BTreeMap<String, IconData>,
    #[serde(default)]
    pub aliases: BTreeMap<String, AliasData>,
    pub left: Option<f64>,
    pub top: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    #[serde(default)]
    pub not_found: Vec<String>,
}

/// An icon ready to be wrapped in an `<svg>` element.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolved {
    Plain {
        body: String,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    },
    /// The icon or one of its aliases rotates or flips it. Rather than
    /// reimplement Iconify's transform rules, callers fetch the SVG the API
    /// renders.
    Transformed,
    Missing,
}

impl IconSetData {
    pub fn resolve(&self, name: &str) -> Resolved {
        let mut left = None;
        let mut top = None;
        let mut width = None;
        let mut height = None;
        let mut transformed = false;
        let mut current = name.to_string();
        // Aliases may point to aliases; a short bound also stops cycles.
        for _ in 0..8 {
            if let Some(icon) = self.icons.get(&current) {
                transformed |= icon.rotate.unwrap_or(0) % 4 != 0
                    || icon.h_flip.unwrap_or(false)
                    || icon.v_flip.unwrap_or(false);
                if transformed {
                    return Resolved::Transformed;
                }
                return Resolved::Plain {
                    body: icon.body.clone(),
                    left: left.or(icon.left).or(self.left).unwrap_or(0.0),
                    top: top.or(icon.top).or(self.top).unwrap_or(0.0),
                    width: width.or(icon.width).or(self.width).unwrap_or(16.0),
                    height: height.or(icon.height).or(self.height).unwrap_or(16.0),
                };
            }
            let Some(alias) = self.aliases.get(&current) else {
                return Resolved::Missing;
            };
            // The alias nearest the requested name wins for dimensions.
            left = left.or(alias.left);
            top = top.or(alias.top);
            width = width.or(alias.width);
            height = height.or(alias.height);
            transformed |= alias.rotate.unwrap_or(0) % 4 != 0
                || alias.h_flip.unwrap_or(false)
                || alias.v_flip.unwrap_or(false);
            current = alias.parent.clone();
        }
        Resolved::Missing
    }
}

impl Resolved {
    /// Wraps a plain icon in a standalone `<svg>` element.
    pub fn to_svg(&self) -> Option<String> {
        match self {
            Resolved::Plain {
                body,
                left,
                top,
                width,
                height,
            } => Some(format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{left} {top} {width} {height}\" width=\"{width}\" height=\"{height}\">{body}</svg>"
            )),
            _ => None,
        }
    }
}

/// Options for `/{prefix}/{name}.svg`.
#[derive(Debug, Clone, Default)]
pub struct SvgOptions {
    /// `#rgb` or `#rrggbb`, replacing `currentColor`.
    pub color: Option<String>,
    pub height: Option<u32>,
    /// `horizontal`, `vertical` or `horizontal,vertical`.
    pub flip: Option<String>,
    /// Quarter turns: 1, 2 or 3.
    pub rotate: Option<u8>,
}

pub struct Client {
    agent: ureq::Agent,
    base: String,
}

impl Client {
    pub fn new(base: &str) -> Result<Self> {
        let base = base.trim().trim_end_matches('/').to_string();
        if !(base.starts_with("https://") || base.starts_with("http://")) {
            bail!("--api must be an http:// or https:// URL, got '{base}'");
        }
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(20)))
            .http_status_as_error(false)
            .user_agent(concat!(
                "iconhunt/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/printemps-tokyo/iconhunt)"
            ))
            .build()
            .new_agent();
        Ok(Client { agent, base })
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    /// The public URL of an icon's SVG, for printing.
    pub fn svg_url(&self, id: &IconId) -> String {
        format!("{}/{}/{}.svg", self.base, id.prefix, id.name)
    }

    fn get(&self, path_and_query: &str, limit: u64) -> Result<String> {
        let url = format!("{}{}", self.base, path_and_query);
        let mut resp = self
            .agent
            .get(&url)
            .call()
            .with_context(|| format!("request to {url} failed"))?;
        let status = resp.status().as_u16();
        match status {
            200 => {}
            404 => bail!("not found: {url}"),
            429 => bail!(
                "the API is rate limiting requests ({url}); wait a while, or run your own Iconify API instance and pass it with --api"
            ),
            _ => bail!("{url} returned HTTP {status}"),
        }
        resp.body_mut()
            .with_config()
            .limit(limit)
            .read_to_string()
            .with_context(|| format!("reading the response from {url}"))
    }

    fn get_json<T: for<'de> Deserialize<'de>>(&self, path_and_query: &str) -> Result<T> {
        let text = self.get(path_and_query, JSON_LIMIT)?;
        serde_json::from_str(&text)
            .with_context(|| format!("unexpected response from {}{path_and_query}", self.base))
    }

    /// All icon sets, or only the given prefixes.
    pub fn collections(&self, prefixes: &[String]) -> Result<BTreeMap<String, SetInfo>> {
        if prefixes.is_empty() {
            self.get_json("/collections")
        } else {
            self.get_json(&format!(
                "/collections?prefixes={}",
                encode(&prefixes.join(","))
            ))
        }
    }

    pub fn search(&self, q: &SearchQuery) -> Result<SearchResponse> {
        let limit = q.limit.clamp(SEARCH_MIN_LIMIT, SEARCH_MAX_LIMIT);
        let mut path = format!("/search?query={}&limit={limit}", encode(&q.query));
        if q.start > 0 {
            path.push_str(&format!("&start={}", q.start));
        }
        if !q.prefixes.is_empty() {
            path.push_str(&format!("&prefixes={}", encode(&q.prefixes.join(","))));
        }
        self.get_json(&path)
    }

    /// Icon data for names in one set, merged across as many requests as the
    /// name list needs.
    pub fn icon_data(&self, prefix: &str, names: &[String]) -> Result<IconSetData> {
        let mut merged = IconSetData::default();
        for chunk in names.chunks(NAMES_PER_REQUEST) {
            let part: IconSetData = self.get_json(&format!(
                "/{prefix}.json?icons={}",
                encode(&chunk.join(","))
            ))?;
            merged.left = merged.left.or(part.left);
            merged.top = merged.top.or(part.top);
            merged.width = merged.width.or(part.width);
            merged.height = merged.height.or(part.height);
            merged.icons.extend(part.icons);
            merged.aliases.extend(part.aliases);
            merged.not_found.extend(part.not_found);
        }
        Ok(merged)
    }

    /// The SVG the API renders for one icon.
    pub fn svg(&self, id: &IconId, opts: &SvgOptions) -> Result<String> {
        let mut params = Vec::new();
        if let Some(color) = &opts.color {
            params.push(format!("color={}", encode(color)));
        }
        if let Some(h) = opts.height {
            params.push(format!("height={h}"));
        }
        if let Some(flip) = &opts.flip {
            params.push(format!("flip={}", encode(flip)));
        }
        if let Some(r) = opts.rotate {
            params.push(format!("rotate={r}"));
        }
        let mut path = format!("/{}/{}.svg", id.prefix, id.name);
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }
        let svg = self.get(&path, SVG_LIMIT)?;
        if !svg.trim_start().starts_with("<svg") {
            bail!("{}{path} did not return an SVG", self.base);
        }
        Ok(svg)
    }

    /// SVGs for many icons: plain icons are built from batched icon data, and
    /// only transformed aliases cost a request of their own. Missing icons map
    /// to `None`.
    pub fn svgs(&self, ids: &[IconId]) -> Result<BTreeMap<IconId, Option<String>>> {
        let mut by_prefix: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for id in ids {
            by_prefix
                .entry(id.prefix.as_str())
                .or_default()
                .push(id.name.clone());
        }
        let mut out = BTreeMap::new();
        for (prefix, names) in by_prefix {
            let data = self.icon_data(prefix, &names)?;
            for name in names {
                let id = IconId {
                    prefix: prefix.to_string(),
                    name: name.clone(),
                };
                let svg = match data.resolve(&name) {
                    Resolved::Missing => None,
                    Resolved::Transformed => Some(self.svg(&id, &SvgOptions::default())?),
                    plain => plain.to_svg(),
                };
                out.insert(id, svg);
            }
        }
        Ok(out)
    }
}

/// Why an SVG is unsafe to save as a file, if it is.
///
/// Icon sets are plain paths, but the response comes from a server, possibly
/// a self-hosted one. A saved SVG opened in a browser runs scripts and loads
/// external resources, so `get` refuses anything that could, rather than
/// trying to clean it.
pub fn svg_hazard(svg: &str) -> Option<&'static str> {
    let lower = svg.to_ascii_lowercase();
    for (needle, reason) in [
        ("<script", "contains a <script> element"),
        ("<foreignobject", "contains a <foreignObject> element"),
        ("<!entity", "declares XML entities"),
        ("<!doctype", "has a DOCTYPE"),
        ("javascript:", "contains a javascript: URL"),
        ("<iframe", "contains an <iframe> element"),
        ("@import", "imports an external stylesheet"),
    ] {
        if lower.contains(needle) {
            return Some(reason);
        }
    }
    let b = lower.as_bytes();
    for i in 0..b.len() {
        let after_space =
            i > 0 && (b[i - 1].is_ascii_whitespace() || b[i - 1] == b'"' || b[i - 1] == b'\'');
        // Event handler attributes: on<letters>=.
        if after_space && b[i..].starts_with(b"on") {
            let mut j = i + 2;
            while j < b.len() && b[j].is_ascii_alphabetic() {
                j += 1;
            }
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j > i + 2 && j < b.len() && b[j] == b'=' {
                return Some("has an event handler attribute");
            }
        }
        // href and url() may only point inside the document (#id).
        let target = if b[i..].starts_with(b"href") {
            let mut j = i + 4;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            (j < b.len() && b[j] == b'=').then_some(j + 1)
        } else if b[i..].starts_with(b"url(") {
            Some(i + 4)
        } else {
            None
        };
        if let Some(mut j) = target {
            while j < b.len() && (b[j].is_ascii_whitespace() || b[j] == b'"' || b[j] == b'\'') {
                j += 1;
            }
            if j < b.len() && b[j] != b'#' {
                return Some("references an external resource");
            }
        }
    }
    None
}

/// Percent-encodes a query component (RFC 3986 unreserved characters pass).
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(json: &str) -> IconSetData {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn encodes_query_components() {
        assert_eq!(encode("arrow left"), "arrow%20left");
        assert_eq!(encode("#ba3329"), "%23ba3329");
        assert_eq!(encode("mdi,mdi-"), "mdi%2Cmdi-");
        assert_eq!(encode("家"), "%E5%AE%B6");
    }

    #[test]
    fn resolves_plain_icons_and_aliases() {
        let data = set(
            r#"{"prefix":"mdi","aliases":{"house":{"parent":"home"},"wide":{"parent":"home","width":32},"deep":{"parent":"wide"}},
                "width":24,"height":24,"icons":{"home":{"body":"<path d=\"M0 0\"/>"}},"not_found":["nope"]}"#,
        );
        let home = data.resolve("home");
        assert_eq!(
            home,
            Resolved::Plain {
                body: "<path d=\"M0 0\"/>".into(),
                left: 0.0,
                top: 0.0,
                width: 24.0,
                height: 24.0
            }
        );
        assert_eq!(data.resolve("house"), home);
        match data.resolve("deep") {
            Resolved::Plain { width, height, .. } => {
                assert_eq!(width, 32.0);
                assert_eq!(height, 24.0);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(data.resolve("nope"), Resolved::Missing);
        assert!(home.to_svg().unwrap().contains("viewBox=\"0 0 24 24\""));
    }

    #[test]
    fn transformed_aliases_and_cycles() {
        let data = set(r#"{"icons":{"a":{"body":"x"},"r":{"body":"x","rotate":1}},
                "aliases":{"flip":{"parent":"a","hFlip":true},"twice":{"parent":"flip"},"r2":{"parent":"r"},
                "loop1":{"parent":"loop2"},"loop2":{"parent":"loop1"}}}"#);
        assert_eq!(data.resolve("flip"), Resolved::Transformed);
        assert_eq!(data.resolve("twice"), Resolved::Transformed);
        assert_eq!(data.resolve("r2"), Resolved::Transformed);
        assert_eq!(data.resolve("loop1"), Resolved::Missing);
        // Defaults when neither icon nor set gives a size.
        match data.resolve("a") {
            Resolved::Plain { width, height, .. } => assert_eq!((width, height), (16.0, 16.0)),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn flags_unsafe_svg() {
        let ok = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><defs><linearGradient id="a"/></defs><path fill="url(#a)" d="M0 0"/><use href="#a"/><g font-weight="bold"/></svg>"##;
        assert_eq!(svg_hazard(ok), None);
        for bad in [
            "<svg><script>alert(1)</script></svg>",
            "<svg onload=\"alert(1)\"></svg>",
            "<svg><a xlink:href = 'javascript:x'>a</a></svg>",
            "<svg><image href=\"https://evil.test/x.png\"/></svg>",
            "<svg><path style=\"fill:url( 'https://evil.test/a')\"/></svg>",
            "<svg><foreignObject/></svg>",
            "<!DOCTYPE svg [<!ENTITY x SYSTEM \"file:///etc/passwd\">]><svg>&x;</svg>",
            "<svg ONCLICK='x'></svg>",
            "<svg><style>@import \"https://evil.test/a.css\";</style></svg>",
        ] {
            assert!(svg_hazard(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn rejects_non_http_base() {
        assert!(Client::new("file:///etc").is_err());
        assert_eq!(
            Client::new("https://api.example.test/").unwrap().base(),
            "https://api.example.test"
        );
    }
}
