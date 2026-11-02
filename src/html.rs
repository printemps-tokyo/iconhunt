//! A self-contained HTML contact sheet of search results.
//!
//! Icons are embedded as `<img src="data:image/svg+xml;base64,...">`: an SVG
//! loaded as an image cannot run scripts or fetch anything, so markup from the
//! API never becomes part of the page.

use crate::license::Class;

pub struct Card {
    pub id: String,
    pub set_name: String,
    pub license: String,
    pub class: Class,
    /// Single-color icon (drawn with `currentColor`), inverted in dark mode.
    pub mono: bool,
    pub svg: Option<String>,
    pub url: String,
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

pub fn page(title: &str, summary: &str, cards: &[Card]) -> String {
    let mut body = String::new();
    for card in cards {
        let img = match &card.svg {
            Some(svg) => format!(
                "<img src=\"data:image/svg+xml;base64,{}\" alt=\"{}\" class=\"{}\">",
                base64(svg.as_bytes()),
                escape(&card.id),
                if card.mono { "mono" } else { "color" }
            ),
            None => "<span class=\"missing\">?</span>".to_string(),
        };
        body.push_str(&format!(
            "<figure class=\"card\">{img}<figcaption><input readonly value=\"{id}\" aria-label=\"Icon id\" onclick=\"this.select()\"><span>{set}</span><span class=\"lic {cls}\">{lic} &middot; {cls}</span><a href=\"{url}\">SVG</a></figcaption></figure>\n",
            id = escape(&card.id),
            set = escape(&card.set_name),
            lic = escape(&card.license),
            cls = card.class.label(),
            url = escape(&card.url),
        ));
    }
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="generator" content="iconhunt {version}">
<title>{title}</title>
<style>
:root {{ --bg: #ffffff; --fg: #1f2328; --muted: #59636e; --line: #d1d9e0; --card: #f6f8fa; --warn: #9a6700; --bad: #cf222e; color-scheme: light dark; }}
@media (prefers-color-scheme: dark) {{
  :root {{ --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --line: #3d444d; --card: #151b23; --warn: #d29922; --bad: #f85149; }}
  img.mono {{ filter: invert(1); }}
}}
body {{ margin: 0; padding: 24px 16px; background: var(--bg); color: var(--fg); font: 14px/1.5 system-ui, sans-serif; }}
h1 {{ font-size: 20px; margin: 0 0 4px; }}
p {{ color: var(--muted); margin: 0 0 20px; }}
main {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 12px; }}
.card {{ margin: 0; padding: 16px 12px; border: 1px solid var(--line); border-radius: 8px; background: var(--card); display: flex; flex-direction: column; align-items: center; gap: 10px; }}
.card img, .missing {{ width: 48px; height: 48px; }}
.missing {{ display: grid; place-items: center; color: var(--muted); font-size: 24px; }}
figcaption {{ display: flex; flex-direction: column; gap: 2px; width: 100%; font-size: 12px; color: var(--muted); text-align: center; }}
input {{ font: 12px ui-monospace, monospace; color: var(--fg); background: transparent; border: 0; text-align: center; width: 100%; }}
.lic.attribution, .lic.unknown {{ color: var(--warn); }}
.lic.copyleft, .lic.non-commercial {{ color: var(--bad); }}
a {{ color: inherit; }}
</style>
</head>
<body>
<h1>{title}</h1>
<p>{summary}</p>
<main>
{body}</main>
</body>
</html>
"#,
        version = env!("CARGO_PKG_VERSION"),
        title = escape(title),
        summary = escape(summary),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (input, expected) in cases {
            assert_eq!(base64(input.as_bytes()), expected);
        }
    }

    #[test]
    fn page_escapes_text_and_never_inlines_svg() {
        let cards = [Card {
            id: "x:<b>".into(),
            set_name: "Set \"A\"".into(),
            license: "MIT".into(),
            class: Class::Permissive,
            mono: true,
            svg: Some("<svg><script>alert(1)</script></svg>".into()),
            url: "https://example.test/x.svg?a=1&b=2".into(),
        }];
        let html = page("<t>", "s & s", &cards);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<b>"));
        assert!(html.contains("x:&lt;b&gt;"));
        assert!(html.contains("a=1&amp;b=2"));
        assert!(html.contains("data:image/svg+xml;base64,"));
        assert!(html.contains("<title>&lt;t&gt;</title>"));
    }
}
