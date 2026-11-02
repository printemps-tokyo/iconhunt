use std::collections::{BTreeMap, BTreeSet};
use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};

use iconhunt::api::{self, Client, Resolved, SearchQuery, SetInfo, SvgOptions};
use iconhunt::id::{check_prefix, IconId};
use iconhunt::license::{classify, Class};
use iconhunt::render::{self, Paint};
use iconhunt::{credits, html, ja};

#[derive(Parser)]
#[command(
    name = "iconhunt",
    version,
    about = "Search open-source icons (Iconify) from the terminal, in English or Japanese",
    long_about = "Search open-source icons (Iconify) from the terminal, in English or Japanese.\n\n\
        Results show each icon set's license as a class (public-domain, permissive, attribution, \
        copyleft, non-commercial, unknown), previews are drawn in the terminal, and `credits` \
        writes the attribution text for the icons you used. License classes are a reading aid, \
        not legal advice.",
    after_help = "Examples:\n  iconhunt search home\n  iconhunt search 家 --no-attribution\n  iconhunt show mdi:home\n  iconhunt get mdi:home lucide:search -o icons/\n  iconhunt credits icons.txt > CREDITS.md\n\n\
        Exit codes: 0 found, 1 nothing found or an icon is missing, 2 error."
)]
struct Cli {
    /// Iconify API base URL (for a self-hosted instance).
    #[arg(long, global = true, env = "ICONHUNT_API", default_value = api::DEFAULT_API)]
    api: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search icons by keyword (English, or Japanese via a built-in table).
    Search(SearchArgs),
    /// Show one icon large, with its set, author and license.
    Show(ShowArgs),
    /// Save icons as SVG or PNG files.
    Get(GetArgs),
    /// List icon sets with their licenses.
    Sets(SetsArgs),
    /// Write Markdown attribution for the icons you used.
    Credits(CreditsArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum Style {
    /// Single-color icons that follow the text color (currentColor).
    Mono,
    /// Icons with their own colors.
    Color,
}

#[derive(Args)]
struct SetFilter {
    /// Only sets whose license is in these classes (comma-separated):
    /// public-domain (pd), permissive, attribution, copyleft, non-commercial, unknown.
    #[arg(long, value_delimiter = ',', value_parser = parse_class)]
    license: Vec<Class>,

    /// Only sets that need no visible credit (public-domain and permissive).
    #[arg(long, conflicts_with = "license")]
    no_attribution: bool,

    /// Only single-color or only multicolor sets.
    #[arg(long, value_enum)]
    style: Option<Style>,

    /// Only sets whose category contains this text (for example "UI 24px", "Emoji", "Logos").
    #[arg(long)]
    category: Option<String>,
}

impl SetFilter {
    fn active(&self) -> bool {
        !self.license.is_empty()
            || self.no_attribution
            || self.style.is_some()
            || self.category.is_some()
    }

    fn accepts(&self, info: &SetInfo) -> bool {
        let class = classify(info.license.spdx.as_deref());
        if !self.license.is_empty() && !self.license.contains(&class) {
            return false;
        }
        if self.no_attribution && !class.no_attribution() {
            return false;
        }
        match self.style {
            Some(Style::Mono) if info.palette => return false,
            Some(Style::Color) if !info.palette => return false,
            _ => {}
        }
        if let Some(cat) = &self.category {
            let have = info.category.as_deref().unwrap_or("").to_lowercase();
            if !have.contains(&cat.to_lowercase()) {
                return false;
            }
        }
        true
    }
}

#[derive(Args)]
struct SearchArgs {
    /// What to look for: `home`, `arrow left`, `家`, `ゴミ箱`.
    #[arg(required = true, num_args = 1..)]
    query: Vec<String>,

    /// Only these icon sets, by prefix (comma-separated), for example mdi,lucide.
    #[arg(short, long = "set", value_delimiter = ',')]
    sets: Vec<String>,

    #[command(flatten)]
    filter: SetFilter,

    /// Number of icons to show (1-999).
    #[arg(short = 'n', long, default_value_t = 24, value_parser = clap::value_parser!(u16).range(1..=999))]
    limit: u16,

    /// Most icons shown from one set, so results span several sets (0 = no cap).
    /// Default 4, or no cap when one --set is given.
    #[arg(long, value_name = "N")]
    per_set: Option<usize>,

    /// Skip this many results (for paging); --start plus --limit is at most 999.
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u16).range(0..=998))]
    start: u16,

    /// Draw previews even when stdout is not a terminal.
    #[arg(long, conflicts_with = "no_preview")]
    preview: bool,

    /// Never draw previews.
    #[arg(long)]
    no_preview: bool,

    /// Preview size in pixels; each terminal cell holds 1x2 pixels.
    #[arg(long, default_value_t = 16, value_parser = clap::value_parser!(u32).range(8..=64))]
    size: u32,

    /// Also write the results as an HTML contact sheet to this file.
    #[arg(long, value_name = "FILE")]
    html: Option<PathBuf>,

    /// Overwrite the --html file if it exists.
    #[arg(long)]
    force: bool,

    /// Send the query as typed, without the Japanese keyword table.
    #[arg(long)]
    no_translate: bool,

    /// Machine-readable output.
    #[arg(long, conflicts_with_all = ["preview"])]
    json: bool,
}

#[derive(Args)]
struct ShowArgs {
    /// Icon id, prefix:name (for example mdi:home).
    id: String,

    /// Preview size in pixels.
    #[arg(long, default_value_t = 32, value_parser = clap::value_parser!(u32).range(8..=128))]
    size: u32,

    /// Leave out the preview.
    #[arg(long)]
    no_preview: bool,

    /// Machine-readable output.
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum Flip {
    H,
    V,
    Hv,
}

#[derive(Args)]
struct GetArgs {
    /// Icon ids, prefix:name.
    #[arg(required = true, num_args = 1..)]
    ids: Vec<String>,

    /// Output file (one icon), directory, or `-` for stdout. Default: the current directory.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Save PNG instead of SVG.
    #[arg(long)]
    png: bool,

    /// Height in pixels (SVG: width/height attributes; PNG: canvas size, default 64).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=4096))]
    size: Option<u32>,

    /// Color for single-color icons, #rgb or #rrggbb.
    #[arg(long, value_parser = parse_color)]
    color: Option<String>,

    /// Flip horizontally (h), vertically (v) or both (hv).
    #[arg(long, value_enum)]
    flip: Option<Flip>,

    /// Rotate clockwise by 90, 180 or 270 degrees.
    #[arg(long, value_parser = ["90", "180", "270"])]
    rotate: Option<String>,

    /// Overwrite existing files.
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct SetsArgs {
    /// Only sets whose prefix or name contains this text.
    text: Option<String>,

    #[command(flatten)]
    filter: SetFilter,

    /// Include sets the API marks hidden (mostly deprecated duplicates).
    #[arg(long)]
    hidden: bool,

    /// Machine-readable output.
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct CreditsArgs {
    /// Icon ids, files with one id per line, or `-` for stdin.
    #[arg(required = true, num_args = 1..)]
    inputs: Vec<String>,

    /// Write to this file instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Overwrite the output file if it exists.
    #[arg(long)]
    force: bool,
}

fn parse_class(s: &str) -> Result<Class, String> {
    Class::parse(s)
}

fn parse_color(s: &str) -> Result<String, String> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if (hex.len() == 3 || hex.len() == 6) && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(format!("#{}", hex.to_ascii_lowercase()))
    } else {
        Err(format!("'{s}' is not a color: use #rgb or #rrggbb"))
    }
}

const DEFAULT_PER_SET: usize = 4;

/// Exit status for "the command ran, but found nothing" (1).
struct NotFound;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = Client::new(&cli.api).and_then(|client| match cli.command {
        Command::Search(a) => search(&client, a),
        Command::Show(a) => show(&client, a),
        Command::Get(a) => get(&client, a),
        Command::Sets(a) => sets(&client, a),
        Command::Credits(a) => credits_cmd(&client, a),
    });
    match result {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(NotFound)) => ExitCode::from(1),
        Err(e) => {
            eprintln!("iconhunt: {e:#}");
            ExitCode::from(2)
        }
    }
}

type Outcome = Result<std::result::Result<(), NotFound>>;

fn color_allowed() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
}

fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .filter(|w| *w > 0)
        .unwrap_or(80)
}

fn paint_for(info: Option<&SetInfo>) -> Paint {
    if info.is_some_and(|i| i.palette) && color_allowed() {
        Paint::TrueColor
    } else {
        Paint::Mask
    }
}

fn license_label(info: &SetInfo) -> String {
    info.license
        .spdx
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| info.license.title.clone())
}

/// Strips control characters from text that came from the API, so a set name
/// cannot carry terminal escape sequences.
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let rows: Vec<Vec<String>> = rows
        .iter()
        .map(|r| r.iter().map(|c| clean(c)).collect())
        .collect();
    let rows = &rows;
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    let line = |cells: Vec<&str>| {
        let mut s = String::new();
        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                s.push_str("  ");
            }
            s.push_str(cell);
            if i + 1 < cells.len() {
                s.push_str(&" ".repeat(widths[i] - cell.chars().count()));
            }
        }
        s.push('\n');
        s
    };
    let mut out = line(headers.to_vec());
    for row in rows {
        out.push_str(&line(row.iter().map(String::as_str).collect()));
    }
    out
}

fn write_new(path: &Path, data: &[u8], force: bool) -> Result<()> {
    if force {
        return std::fs::write(path, data).with_context(|| format!("writing {}", path.display()));
    }
    // create_new fails on anything already there, a dangling symlink included,
    // and leaves no gap between the check and the write.
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            bail!("{} exists; pass --force to overwrite it", path.display())
        }
        Err(e) => return Err(e).with_context(|| format!("creating {}", path.display())),
    };
    file.write_all(data)
        .with_context(|| format!("writing {}", path.display()))
}

/// Prefixes to search in, empty for all sets, or `None` when no set passes the
/// filters.
fn search_prefixes(
    client: &Client,
    sets: &[String],
    filter: &SetFilter,
) -> Result<Option<Vec<String>>> {
    for p in sets {
        check_prefix(p).map_err(anyhow::Error::msg)?;
    }
    if !filter.active() {
        return Ok(Some(sets.to_vec()));
    }
    let all = client.collections(sets)?;
    let allowed: Vec<String> = all
        .iter()
        .filter(|(_, info)| !info.hidden && filter.accepts(info))
        .map(|(p, _)| p.clone())
        .collect();
    Ok((!allowed.is_empty()).then_some(allowed))
}

fn search(client: &Client, a: SearchArgs) -> Outcome {
    let typed = a.query.join(" ");
    let translation = if a.no_translate {
        ja::Translation {
            queries: vec![typed.trim().to_string()],
            ..Default::default()
        }
    } else {
        ja::translate(&typed)
    };
    if !translation.unknown.is_empty() {
        eprintln!(
            "iconhunt: no English keyword known for {}; Iconify only matches English names, so try an English word",
            translation
                .unknown
                .iter()
                .map(|u| format!("'{u}'"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if translation.queries.is_empty() || translation.queries[0].is_empty() {
        return Ok(Err(NotFound));
    }
    if translation.translated() && !a.json {
        let parts: Vec<String> = translation
            .mapped
            .iter()
            .map(|(word, en)| format!("{word} -> {}", en.join(", ")))
            .collect();
        eprintln!(
            "iconhunt: {} (searching: {})",
            parts.join("; "),
            translation
                .queries
                .iter()
                .map(|q| format!("\"{q}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let Some(prefixes) = search_prefixes(client, &a.sets, &a.filter)? else {
        eprintln!("iconhunt: no icon set matches the filters");
        return Ok(Err(NotFound));
    };
    let limit = a.limit as usize;
    let per_set = a.per_set.unwrap_or(if a.sets.len() == 1 {
        0
    } else {
        DEFAULT_PER_SET
    });
    let start = a.start as usize;
    // Both are at most 999, so neither sum nor product can overflow.
    let want = limit + start;
    if want > api::SEARCH_MAX_LIMIT {
        bail!(
            "--start plus --limit is {want}; the search API returns at most {} results",
            api::SEARCH_MAX_LIMIT
        );
    }
    // A per-set cap throws results away, so ask for more of them.
    let fetch = if per_set > 0 { want * 6 } else { want }.min(api::SEARCH_MAX_LIMIT);
    let mut candidates: Vec<IconId> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut infos: BTreeMap<String, SetInfo> = BTreeMap::new();
    for q in &translation.queries {
        if per_set == 0 && candidates.len() >= want {
            break;
        }
        let resp = client.search(&SearchQuery {
            query: q.clone(),
            limit: fetch,
            start: 0,
            prefixes: prefixes.clone(),
        })?;
        infos.extend(resp.collections);
        for name in &resp.icons {
            let Ok(id) = IconId::parse(name) else {
                continue;
            };
            if seen.insert(id.clone()) {
                candidates.push(id);
            }
        }
    }
    let total = candidates.len();
    let capped = spread(candidates, per_set);
    if capped.len() < total && !a.json {
        eprintln!(
            "iconhunt: showing at most {per_set} icons per set; --per-set 0 shows all, --set PREFIX shows one set"
        );
    }
    let ids: Vec<IconId> = capped.into_iter().skip(start).take(limit).collect();
    if ids.is_empty() {
        if !a.json {
            eprintln!("iconhunt: no icons found");
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&search_json(
                    &typed,
                    &translation,
                    &[],
                    &infos,
                    client
                ))?
            );
        }
        return Ok(Err(NotFound));
    }

    if a.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&search_json(&typed, &translation, &ids, &infos, client))?
        );
        return Ok(Ok(()));
    }

    let want_preview = !a.no_preview && (a.preview || std::io::stdout().is_terminal());
    let svgs = if want_preview || a.html.is_some() {
        client.svgs(&ids)?
    } else {
        BTreeMap::new()
    };

    let mut out = String::new();
    if want_preview {
        let cells: Vec<Vec<String>> = ids
            .iter()
            .map(|id| {
                let blank = vec![" ".repeat(a.size as usize); (a.size as usize).div_ceil(2)];
                svgs.get(id)
                    .and_then(|s| s.as_deref())
                    .and_then(|svg| render::rasterize(svg, a.size).ok())
                    .map(|pix| render::terminal_lines(&pix, paint_for(infos.get(&id.prefix))))
                    .unwrap_or(blank)
            })
            .collect();
        out.push_str(&render::grid(&cells, a.size as usize, terminal_width()));
    }
    let rows: Vec<Vec<String>> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let info = infos.get(&id.prefix).cloned().unwrap_or_default();
            vec![
                (i + 1).to_string(),
                id.to_string(),
                info.name.clone(),
                license_label(&info),
                classify(info.license.spdx.as_deref()).label().to_string(),
            ]
        })
        .collect();
    out.push_str(&table(&["#", "icon", "set", "license", "class"], &rows));
    std::io::stdout().write_all(out.as_bytes())?;

    if let Some(path) = &a.html {
        let cards: Vec<html::Card> = ids
            .iter()
            .map(|id| {
                let info = infos.get(&id.prefix).cloned().unwrap_or_default();
                html::Card {
                    id: id.to_string(),
                    set_name: info.name.clone(),
                    license: license_label(&info),
                    class: classify(info.license.spdx.as_deref()),
                    mono: !info.palette,
                    svg: svgs.get(id).cloned().flatten(),
                    url: client.svg_url(id),
                }
            })
            .collect();
        let summary = format!(
            "{} icons for \"{}\" from the Iconify API",
            ids.len(),
            translation.queries.join("\", \"")
        );
        let page = html::page(&format!("iconhunt: {typed}"), &summary, &cards);
        write_new(path, page.as_bytes(), a.force)?;
        eprintln!("iconhunt: wrote {}", path.display());
    }
    Ok(Ok(()))
}

/// Caps each set at `per_set` icons (0 = no cap) and interleaves the sets:
/// the first match of every set, then the second, and so on, so different
/// designs of the same icon sit next to each other. Sets keep the order in
/// which the API first returned them.
fn spread(ids: Vec<IconId>, per_set: usize) -> Vec<IconId> {
    if per_set == 0 {
        return ids;
    }
    let mut order: Vec<String> = Vec::new();
    let mut groups: BTreeMap<String, Vec<IconId>> = BTreeMap::new();
    for id in ids {
        let group = groups.entry(id.prefix.clone()).or_default();
        if group.is_empty() {
            order.push(id.prefix.clone());
        }
        if group.len() < per_set {
            group.push(id);
        }
    }
    let rounds = groups.values().map(Vec::len).max().unwrap_or(0);
    let mut out = Vec::new();
    for round in 0..rounds {
        for prefix in &order {
            if let Some(id) = groups[prefix].get(round) {
                out.push(id.clone());
            }
        }
    }
    out
}

fn icon_json(id: &IconId, info: Option<&SetInfo>, client: &Client) -> Value {
    let info = info.cloned().unwrap_or_default();
    json!({
        "id": id.to_string(),
        "prefix": id.prefix,
        "name": id.name,
        "set": info.name,
        "author": info.author,
        "license": info.license,
        "class": classify(info.license.spdx.as_deref()),
        "palette": info.palette,
        "svg_url": client.svg_url(id),
    })
}

fn search_json(
    typed: &str,
    t: &ja::Translation,
    ids: &[IconId],
    infos: &BTreeMap<String, SetInfo>,
    client: &Client,
) -> Value {
    json!({
        "query": typed,
        "queries": t.queries,
        "mapped": t.mapped.iter().map(|(w, en)| json!({"word": w, "keywords": en})).collect::<Vec<_>>(),
        "unknown": t.unknown,
        "icons": ids.iter().map(|id| icon_json(id, infos.get(&id.prefix), client)).collect::<Vec<_>>(),
    })
}

fn show(client: &Client, a: ShowArgs) -> Outcome {
    let id = IconId::parse(&a.id).map_err(anyhow::Error::msg)?;
    let sets = client.collections(std::slice::from_ref(&id.prefix))?;
    let Some(info) = sets.get(&id.prefix) else {
        eprintln!(
            "iconhunt: unknown icon set '{}'; `iconhunt sets` lists them",
            clean(&id.prefix)
        );
        return Ok(Err(NotFound));
    };
    let data = client.icon_data(&id.prefix, std::slice::from_ref(&id.name))?;
    let resolved = data.resolve(&id.name);
    let svg = match &resolved {
        Resolved::Missing => {
            eprintln!("iconhunt: {id} not found");
            let near = client.search(&SearchQuery {
                query: id.name.replace('-', " "),
                limit: api::SEARCH_MIN_LIMIT,
                start: 0,
                prefixes: vec![id.prefix.clone()],
            });
            if let Ok(near) = near {
                if !near.icons.is_empty() {
                    let list: Vec<String> = near
                        .icons
                        .iter()
                        .filter_map(|s| IconId::parse(s).ok())
                        .take(5)
                        .map(|id| id.to_string())
                        .collect();
                    eprintln!("iconhunt: similar: {}", list.join(", "));
                }
            }
            return Ok(Err(NotFound));
        }
        Resolved::Transformed => client.svg(&id, &SvgOptions::default())?,
        plain => plain.to_svg().unwrap_or_default(),
    };
    let class = classify(info.license.spdx.as_deref());

    if a.json {
        let mut v = icon_json(&id, Some(info), client);
        if let Resolved::Plain { width, height, .. } = &resolved {
            v["width"] = json!(width);
            v["height"] = json!(height);
        }
        v["svg"] = json!(svg);
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(Ok(()));
    }

    let mut out = String::new();
    if !a.no_preview {
        let pix = render::rasterize(&svg, a.size)?;
        let lines = render::terminal_lines(&pix, paint_for(Some(info)));
        let first = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
        let last = lines
            .iter()
            .rposition(|l| !l.trim().is_empty())
            .map_or(0, |i| i + 1);
        for line in lines.get(first..last).unwrap_or_default() {
            out.push_str("  ");
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out.push('\n');
    }
    let mut rows: Vec<(&str, String)> = vec![("icon", id.to_string())];
    let mut set = info.name.clone();
    if let Some(total) = info.total {
        set.push_str(&format!(" ({total} icons)"));
    }
    rows.push(("set", set));
    if let Some(cat) = &info.category {
        rows.push(("category", cat.clone()));
    }
    let mut author = info.author.name.clone();
    if let Some(url) = &info.author.url {
        author.push_str(&format!(" <{url}>"));
    }
    rows.push(("author", author));
    let mut license = license_label(info);
    if let Some(url) = &info.license.url {
        license.push_str(&format!(" <{url}>"));
    }
    rows.push(("license", license));
    rows.push(("class", format!("{} - {}", class.label(), class.note())));
    rows.push((
        "style",
        if info.palette {
            "multicolor"
        } else {
            "single color (currentColor)"
        }
        .to_string(),
    ));
    if let Resolved::Plain { width, height, .. } = &resolved {
        rows.push(("viewBox", format!("{width} x {height}")));
    }
    rows.push(("svg", client.svg_url(&id)));
    rows.push((
        "html",
        format!("<iconify-icon icon=\"{id}\"></iconify-icon>"),
    ));
    for (k, v) in rows {
        out.push_str(&format!("{k:<9} {}\n", clean(&v)));
    }
    std::io::stdout().write_all(out.as_bytes())?;
    Ok(Ok(()))
}

fn get(client: &Client, a: GetArgs) -> Outcome {
    let ids: Vec<IconId> = a
        .ids
        .iter()
        .map(|s| IconId::parse(s).map_err(anyhow::Error::msg))
        .collect::<Result<_>>()?;
    let ext = if a.png { "png" } else { "svg" };
    let to_stdout = a.output.as_deref() == Some(Path::new("-"));
    if to_stdout && ids.len() > 1 {
        bail!("-o - writes one icon to stdout; got {}", ids.len());
    }
    if to_stdout && a.png && std::io::stdout().is_terminal() {
        bail!("refusing to write PNG bytes to a terminal; redirect stdout or use -o FILE");
    }
    // One icon and an -o that is not an existing directory names the file.
    let out_file = match &a.output {
        Some(p)
            if !to_stdout
                && ids.len() == 1
                && !p.is_dir()
                && !p.to_string_lossy().ends_with('/') =>
        {
            Some(p.clone())
        }
        _ => None,
    };
    let out_dir = match (&a.output, &out_file) {
        (_, Some(_)) => None,
        (Some(p), None) if !to_stdout => Some(p.clone()),
        _ => None,
    };
    if let Some(dir) = &out_dir {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }

    let prefixes: Vec<String> = ids
        .iter()
        .map(|i| i.prefix.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let sets = client.collections(&prefixes)?;
    let opts = SvgOptions {
        color: a.color.clone(),
        height: if a.png { None } else { a.size },
        flip: a.flip.map(|f| {
            match f {
                Flip::H => "horizontal",
                Flip::V => "vertical",
                Flip::Hv => "horizontal,vertical",
            }
            .to_string()
        }),
        rotate: a.rotate.as_deref().map(|r| match r {
            "90" => 1,
            "180" => 2,
            _ => 3,
        }),
    };

    let mut missing = Vec::new();
    let mut needs_credit = Vec::new();
    for id in &ids {
        let Some(info) = sets.get(&id.prefix) else {
            eprintln!("iconhunt: unknown icon set '{}' ({id})", id.prefix);
            missing.push(id.clone());
            continue;
        };
        let svg = match client.svg(id, &opts) {
            Ok(svg) => svg,
            Err(e) if e.to_string().starts_with("not found") => {
                eprintln!("iconhunt: {id} not found");
                missing.push(id.clone());
                continue;
            }
            Err(e) => return Err(e),
        };
        if let Some(reason) = api::svg_hazard(&svg) {
            bail!("refusing to save {id}: the SVG from the API {reason}");
        }
        let bytes = if a.png {
            render::to_png(&svg, a.size.unwrap_or(64))?
        } else {
            svg.into_bytes()
        };
        let class = classify(info.license.spdx.as_deref());
        if !class.no_attribution() {
            needs_credit.push(id.to_string());
        }
        if to_stdout {
            std::io::stdout().write_all(&bytes)?;
            continue;
        }
        let path = match (&out_file, &out_dir) {
            (Some(f), _) => f.clone(),
            (None, Some(d)) => d.join(format!("{}.{ext}", id.file_stem())),
            (None, None) => PathBuf::from(format!("{}.{ext}", id.file_stem())),
        };
        write_new(&path, &bytes, a.force)?;
        eprintln!(
            "iconhunt: saved {} ({}, {}: {})",
            path.display(),
            clean(&info.name),
            clean(&license_label(info)),
            class.label()
        );
    }
    if !needs_credit.is_empty() {
        eprintln!(
            "iconhunt: some icons need credit or carry extra terms; `iconhunt credits {}` writes the text",
            needs_credit.join(" ")
        );
    }
    if missing.is_empty() {
        Ok(Ok(()))
    } else {
        Ok(Err(NotFound))
    }
}

fn sets(client: &Client, a: SetsArgs) -> Outcome {
    let all = client.collections(&[])?;
    let needle = a.text.as_deref().map(str::to_lowercase);
    let picked: Vec<(&String, &SetInfo)> = all
        .iter()
        .filter(|(_, info)| a.hidden || !info.hidden)
        .filter(|(p, info)| {
            needle
                .as_deref()
                .is_none_or(|n| p.contains(n) || info.name.to_lowercase().contains(n))
        })
        .filter(|(_, info)| a.filter.accepts(info))
        .collect();
    if a.json {
        let list: Vec<Value> = picked
            .iter()
            .map(|(p, info)| {
                let mut v = serde_json::to_value(info).unwrap_or(Value::Null);
                v["prefix"] = json!(p);
                v["class"] = json!(classify(info.license.spdx.as_deref()));
                v
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else if !picked.is_empty() {
        let rows: Vec<Vec<String>> = picked
            .iter()
            .map(|(p, info)| {
                vec![
                    p.to_string(),
                    info.name.clone(),
                    info.total.map(|t| t.to_string()).unwrap_or_default(),
                    license_label(info),
                    classify(info.license.spdx.as_deref()).label().to_string(),
                    if info.palette { "color" } else { "mono" }.to_string(),
                    info.category.clone().unwrap_or_default(),
                ]
            })
            .collect();
        print!(
            "{}",
            table(
                &["prefix", "name", "icons", "license", "class", "style", "category"],
                &rows
            )
        );
        eprintln!("iconhunt: {} of {} icon sets", picked.len(), all.len());
    }
    if picked.is_empty() {
        if !a.json {
            eprintln!("iconhunt: no icon set matches");
        }
        return Ok(Err(NotFound));
    }
    Ok(Ok(()))
}

fn read_ids(inputs: &[String]) -> Result<Vec<IconId>> {
    let mut text = String::new();
    for input in inputs {
        if input == "-" {
            std::io::stdin()
                .read_to_string(&mut text)
                .context("reading stdin")?;
            text.push('\n');
        } else if input.contains(':') && !Path::new(input).exists() {
            text.push_str(input);
            text.push('\n');
        } else {
            let content =
                std::fs::read_to_string(input).with_context(|| format!("reading {input}"))?;
            text.push_str(&content);
            text.push('\n');
        }
    }
    let mut ids = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("");
        for word in line.split(|c: char| c.is_whitespace() || c == ',') {
            if word.is_empty() {
                continue;
            }
            let id = IconId::parse(word).map_err(|e| anyhow::anyhow!("line {}: {e}", n + 1))?;
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    if ids.is_empty() {
        bail!("no icon ids given");
    }
    Ok(ids)
}

fn credits_cmd(client: &Client, a: CreditsArgs) -> Outcome {
    let ids = read_ids(&a.inputs)?;
    let prefixes: Vec<String> = ids
        .iter()
        .map(|i| i.prefix.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let sets = client.collections(&prefixes)?;
    let mut problems = Vec::new();
    let mut found: Vec<IconId> = Vec::new();
    for p in &prefixes {
        if !sets.contains_key(p) {
            problems.push(format!("unknown icon set '{p}'"));
            continue;
        }
        let names: Vec<String> = ids
            .iter()
            .filter(|i| &i.prefix == p)
            .map(|i| i.name.clone())
            .collect();
        let data = client.icon_data(p, &names)?;
        for name in names {
            if data.resolve(&name) == Resolved::Missing {
                problems.push(format!("{p}:{name} not found"));
            } else {
                found.push(IconId {
                    prefix: p.clone(),
                    name,
                });
            }
        }
    }
    for p in &problems {
        eprintln!("iconhunt: {p}");
    }
    // Keep the order the ids were given in.
    let found: Vec<IconId> = ids.into_iter().filter(|id| found.contains(id)).collect();
    if found.is_empty() {
        return Ok(Err(NotFound));
    }
    let md = credits::markdown(&found, &sets);
    match &a.output {
        Some(path) => {
            write_new(path, md.as_bytes(), a.force)?;
            eprintln!("iconhunt: wrote {}", path.display());
        }
        None => print!("{md}"),
    }
    if problems.is_empty() {
        Ok(Ok(()))
    } else {
        Ok(Err(NotFound))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<IconId> {
        list.iter().map(|s| IconId::parse(s).unwrap()).collect()
    }

    #[test]
    fn spread_caps_and_interleaves_sets() {
        let input = ids(&["a:1", "a:2", "a:3", "b:1", "c:1", "b:2"]);
        let out = spread(input.clone(), 2);
        assert_eq!(out, ids(&["a:1", "b:1", "c:1", "a:2", "b:2"]));
        assert_eq!(spread(input.clone(), 0), input);
    }

    #[test]
    fn colors_are_normalized() {
        assert_eq!(parse_color("F00").unwrap(), "#f00");
        assert_eq!(parse_color("#BA3329").unwrap(), "#ba3329");
        assert!(parse_color("#12345").is_err());
        assert!(parse_color("red").is_err());
    }
}
