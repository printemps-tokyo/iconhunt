//! A tiny stand-in for the Iconify API, so the CLI tests never touch the
//! network. Responses are shortened copies of real api.iconify.design output.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Server {
    pub base: String,
    pub requests: Arc<Mutex<Vec<String>>>,
}

impl Server {
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

const MDI: &str = r#""mdi":{"name":"Material Design Icons","total":7447,"author":{"name":"Pictogrammers","url":"https://github.com/Templarian/MaterialDesign"},"license":{"title":"Apache 2.0","spdx":"Apache-2.0","url":"https://github.com/Templarian/MaterialDesign/blob/master/LICENSE"},"category":"Material","palette":false}"#;
const GAME: &str = r#""game-icons":{"name":"Game Icons","total":4123,"author":{"name":"GameIcons","url":"https://github.com/game-icons/icons"},"license":{"title":"CC BY 3.0","spdx":"CC-BY-3.0","url":"https://github.com/game-icons/icons/blob/master/license.txt"},"category":"Thematic","palette":false}"#;
const EMOJI: &str = r#""fluent-emoji-flat":{"name":"Fluent Emoji Flat","total":3145,"author":{"name":"Microsoft Corporation","url":"https://github.com/microsoft/fluentui-emoji"},"license":{"title":"MIT","spdx":"MIT","url":"https://github.com/microsoft/fluentui-emoji/blob/main/LICENSE"},"category":"Emoji","palette":true}"#;
const NC: &str = r#""cbi":{"name":"Custom Brand Icons","total":1790,"author":{"name":"Emanuele & rchiileea"},"license":{"title":"CC BY-NC-SA 4.0","spdx":"CC-BY-NC-SA-4.0"},"category":"Logos","palette":false}"#;

const HOME_BODY: &str =
    r#"<path fill=\"currentColor\" d=\"M10 20v-6h4v6h5v-8h3L12 3L2 12h3v8z\"/>"#;
const HOME_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M10 20v-6h4v6h5v-8h3L12 3L2 12h3v8z"/></svg>"#;

fn params(query: &str) -> BTreeMap<String, String> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k.to_string(), decode(v)))
        .collect()
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
            out.push(u8::from_str_radix(hex, 16).unwrap());
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

fn search(p: &BTreeMap<String, String>) -> String {
    let all: &[(&str, &str)] = &[
        ("home", "mdi:home"),
        ("home", "mdi:home-outline"),
        ("home", "mdi:house"),
        ("home", "game-icons:house"),
        ("home", "cbi:home"),
        ("house", "game-icons:house"),
        ("house", "mdi:house"),
        ("cat", "fluent-emoji-flat:cat"),
    ];
    let q = p.get("query").map(String::as_str).unwrap_or("");
    let allowed: Option<Vec<&str>> = p.get("prefixes").map(|s| s.split(',').collect());
    let icons: Vec<String> = all
        .iter()
        .filter(|(k, _)| *k == q)
        .filter(|(_, id)| {
            let prefix = id.split(':').next().unwrap();
            allowed.as_ref().is_none_or(|a| a.contains(&prefix))
        })
        .map(|(_, id)| format!("\"{id}\""))
        .collect();
    let mut sets = Vec::new();
    for (prefix, json) in [
        ("mdi", MDI),
        ("game-icons", GAME),
        ("cbi", NC),
        ("fluent-emoji-flat", EMOJI),
    ] {
        if icons.iter().any(|i| i.starts_with(&format!("\"{prefix}:"))) {
            sets.push(json);
        }
    }
    format!(
        r#"{{"icons":[{}],"total":{},"limit":32,"start":0,"collections":{{{}}},"request":{{}}}}"#,
        icons.join(","),
        icons.len(),
        sets.join(",")
    )
}

fn collections(p: &BTreeMap<String, String>) -> String {
    let all = [
        ("mdi", MDI),
        ("game-icons", GAME),
        ("cbi", NC),
        ("fluent-emoji-flat", EMOJI),
    ];
    let wanted: Option<Vec<&str>> = p.get("prefixes").map(|s| s.split(',').collect());
    let picked: Vec<&str> = all
        .iter()
        .filter(|(prefix, _)| wanted.as_ref().is_none_or(|w| w.contains(prefix)))
        .map(|(_, json)| *json)
        .collect();
    format!("{{{}}}", picked.join(","))
}

/// Icons as (name, JSON member) and aliases as (name, JSON member).
type Fixture<'a> = (&'a [(&'a str, String)], &'a [(&'a str, &'a str)]);

fn icon_data(prefix: &str, p: &BTreeMap<String, String>) -> Option<String> {
    let names: Vec<&str> = p.get("icons")?.split(',').collect();
    let (known, aliases): Fixture = match prefix {
        "mdi" => (
            &[
                ("home", format!(r#""home":{{"body":"{HOME_BODY}"}}"#)),
                (
                    "home-outline",
                    format!(r#""home-outline":{{"body":"{HOME_BODY}"}}"#),
                ),
            ],
            &[("house", r#""house":{"parent":"home"}"#)],
        ),
        "game-icons" => (
            &[("house", format!(r#""house":{{"body":"{HOME_BODY}"}}"#))],
            &[],
        ),
        "cbi" => (
            &[("home", format!(r#""home":{{"body":"{HOME_BODY}"}}"#))],
            &[],
        ),
        "fluent-emoji-flat" => (
            &[(
                "cat",
                r##""cat":{"body":"<path fill=\"#ff8a00\" d=\"M0 0h32v32H0z\"/>"}"##.to_string(),
            )],
            &[],
        ),
        _ => return None,
    };
    let mut icons = Vec::new();
    let mut alias_out = Vec::new();
    let mut missing = Vec::new();
    for name in names {
        if let Some((_, json)) = aliases.iter().find(|(n, _)| *n == name) {
            alias_out.push(json.to_string());
            if let Some((_, parent)) = known.iter().find(|(n, _)| *n == "home") {
                icons.push(parent.clone());
            }
        } else if let Some((_, json)) = known.iter().find(|(n, _)| *n == name) {
            icons.push(json.clone());
        } else {
            missing.push(format!("\"{name}\""));
        }
    }
    icons.dedup();
    let size = if prefix == "fluent-emoji-flat" {
        32
    } else {
        24
    };
    let mut out = format!(
        r#"{{"prefix":"{prefix}","width":{size},"height":{size},"icons":{{{}}},"aliases":{{{}}}"#,
        icons.join(","),
        alias_out.join(",")
    );
    if !missing.is_empty() {
        out.push_str(&format!(r#","not_found":[{}]"#, missing.join(",")));
    }
    out.push('}');
    Some(out)
}

fn route(path: &str, query: &str) -> (u16, &'static str, String) {
    let p = params(query);
    if path == "/search" {
        return (200, "application/json", search(&p));
    }
    if path == "/collections" {
        return (200, "application/json", collections(&p));
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [file] if file.ends_with(".json") => match icon_data(file.trim_end_matches(".json"), &p) {
            Some(body) => (200, "application/json", body),
            None => (404, "text/plain", "404".into()),
        },
        ["mdi", "home.svg"] => {
            let mut svg = HOME_SVG.to_string();
            if let Some(c) = p.get("color") {
                svg = svg.replace("currentColor", c);
            }
            (200, "image/svg+xml", svg)
        }
        ["mdi", "evil.svg"] => (
            200,
            "image/svg+xml",
            r#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"><path d="M0 0"/></svg>"#
                .into(),
        ),
        _ => (404, "text/plain", "404".into()),
    }
}

pub fn start() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let log = requests.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            // Drain the headers.
            loop {
                let mut h = String::new();
                if reader.read_line(&mut h).is_err() || h == "\r\n" || h.is_empty() {
                    break;
                }
            }
            let target = line.split_whitespace().nth(1).unwrap_or("/").to_string();
            log.lock().unwrap().push(decode(&target));
            let (path, query) = target.split_once('?').unwrap_or((&target, ""));
            let (status, kind, body) = route(path, query);
            let reason = if status == 200 { "OK" } else { "Not Found" };
            let _ = write!(
                stream,
                "HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    Server { base, requests }
}

pub fn run(server: &Server, args: &[&str], dir: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_iconhunt"))
        .args(args)
        .env("ICONHUNT_API", &server.base)
        .env_remove("NO_COLOR")
        .current_dir(dir)
        .output()
        .unwrap()
}

pub fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

pub fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}
