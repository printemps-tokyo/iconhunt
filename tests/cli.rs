mod common;

use common::{run, start, stderr, stdout};

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn search_lists_icons_with_license_classes() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["search", "home", "--per-set", "0"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("mdi:home "), "{out}");
    assert!(out.contains("game-icons:house"), "{out}");
    assert!(out.contains("attribution"), "{out}");
    assert!(out.contains("non-commercial"), "{out}");
    // Not a terminal: no preview grid.
    assert!(!out.contains("[1]"), "{out}");
}

#[test]
fn japanese_query_is_mapped_to_english() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["search", "いえ"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    assert!(stderr(&o).contains("イエ -> home, house"), "{}", stderr(&o));
    let reqs = s.requests();
    assert!(
        reqs.iter().any(|r| r.contains("/search?query=home&")),
        "{reqs:?}"
    );
    assert!(stdout(&o).contains("mdi:home"));
}

#[test]
fn unknown_japanese_exits_1_without_a_request() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["search", "ぴよぴよ"], dir.path());
    assert_eq!(o.status.code(), Some(1));
    assert!(
        stderr(&o).contains("no English keyword known"),
        "{}",
        stderr(&o)
    );
    assert!(s.requests().is_empty());
}

#[test]
fn license_filter_restricts_sets_on_the_server() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["search", "home", "--no-attribution"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("mdi:home"));
    assert!(!out.contains("game-icons"), "{out}");
    assert!(!out.contains("cbi:"), "{out}");
    let reqs = s.requests();
    let search = reqs.iter().find(|r| r.starts_with("/search")).unwrap();
    assert!(search.contains("prefixes="), "{search}");
    assert!(!search.contains("game-icons"), "{search}");
}

#[test]
fn per_set_cap_interleaves() {
    let s = start();
    let dir = tmp();
    let o = run(
        &s,
        &["search", "home", "--per-set", "1", "--json"],
        dir.path(),
    );
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    let ids: Vec<&str> = v["icons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["mdi:home", "game-icons:house", "cbi:home"]);
    assert_eq!(v["icons"][1]["class"], "attribution");
    assert_eq!(v["icons"][0]["license"]["spdx"], "Apache-2.0");
}

#[test]
fn preview_grid_draws_blocks() {
    let s = start();
    let dir = tmp();
    let o = run(
        &s,
        &["search", "cat", "--preview", "--size", "8"],
        dir.path(),
    );
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let out = stdout(&o);
    // A filled square: four rows of full blocks, then the label.
    assert!(out.contains("████████\n"), "{out}");
    assert!(out.contains("[1]"), "{out}");
}

#[test]
fn html_sheet_embeds_images() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["search", "home", "--html", "sheet.html"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let html = std::fs::read_to_string(dir.path().join("sheet.html")).unwrap();
    assert!(html.contains("data:image/svg+xml;base64,"));
    assert!(html.contains("mdi:home"));
    let again = run(&s, &["search", "home", "--html", "sheet.html"], dir.path());
    assert_eq!(again.status.code(), Some(2));
    assert!(stderr(&again).contains("--force"));
}

#[test]
fn show_prints_license_and_suggests_on_miss() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["show", "mdi:house", "--size", "16"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("Apache-2.0"), "{out}");
    assert!(out.contains("permissive"), "{out}");
    assert!(out.contains("viewBox   24 x 24"), "{out}");
    assert!(out.contains('█'), "{out}");

    let miss = run(&s, &["show", "mdi:hom"], dir.path());
    assert_eq!(miss.status.code(), Some(1));
    assert!(
        stderr(&miss).contains("mdi:hom not found"),
        "{}",
        stderr(&miss)
    );
}

#[test]
fn get_saves_svg_and_png_and_refuses_overwrite() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["get", "mdi:home", "--color", "#BA3329"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let svg = std::fs::read_to_string(dir.path().join("mdi-home.svg")).unwrap();
    assert!(svg.contains("#ba3329"), "{svg}");
    assert!(s.requests().iter().any(|r| r.contains("color=#ba3329")));

    let again = run(&s, &["get", "mdi:home"], dir.path());
    assert_eq!(again.status.code(), Some(2));

    let png = run(
        &s,
        &["get", "mdi:home", "--png", "--size", "20", "-o", "out/"],
        dir.path(),
    );
    assert_eq!(png.status.code(), Some(0), "{}", stderr(&png));
    let bytes = std::fs::read(dir.path().join("out/mdi-home.png")).unwrap();
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");

    let missing = run(&s, &["get", "mdi:nope"], dir.path());
    assert_eq!(missing.status.code(), Some(1));
}

#[test]
fn credits_groups_sets_and_skips_missing_icons() {
    let s = start();
    let dir = tmp();
    std::fs::write(
        dir.path().join("used.txt"),
        "mdi:home  # header\n\ngame-icons:house, mdi:nope\n",
    )
    .unwrap();
    let o = run(&s, &["credits", "used.txt"], dir.path());
    assert_eq!(o.status.code(), Some(1));
    let md = stdout(&o);
    assert!(
        md.find("## Game Icons").unwrap() < md.find("## Material Design Icons").unwrap(),
        "{md}"
    );
    assert!(md.contains("`mdi:home`"));
    assert!(!md.contains("nope"), "{md}");
    assert!(stderr(&o).contains("mdi:nope not found"));

    let ok = run(&s, &["credits", "mdi:home"], dir.path());
    assert_eq!(ok.status.code(), Some(0), "{}", stderr(&ok));
}

#[test]
fn sets_filters_by_license() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["sets", "--license", "nc"], dir.path());
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("cbi"), "{out}");
    assert!(!out.contains("mdi"), "{out}");

    let none = run(&s, &["sets", "zzz"], dir.path());
    assert_eq!(none.status.code(), Some(1));
}

#[test]
fn bad_input_is_an_error() {
    let s = start();
    let dir = tmp();
    for args in [
        &["show", "mdi:../x"][..],
        &["get", "mdi:home", "--color", "red"],
        &["--api", "ftp://example.test", "sets"],
    ] {
        let o = run(&s, args, dir.path());
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", stderr(&o));
    }
}

#[test]
fn get_refuses_unsafe_svg_and_dangling_symlinks() {
    let s = start();
    let dir = tmp();
    let o = run(&s, &["get", "mdi:evil"], dir.path());
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("event handler"), "{}", stderr(&o));
    assert!(!dir.path().join("mdi-evil.svg").exists());

    #[cfg(unix)]
    {
        let target = dir.path().join("elsewhere.svg");
        std::os::unix::fs::symlink(&target, dir.path().join("link.svg")).unwrap();
        let o = run(&s, &["get", "mdi:home", "-o", "link.svg"], dir.path());
        assert_eq!(o.status.code(), Some(2), "{}", stderr(&o));
        assert!(!target.exists());
    }
}

#[test]
fn not_found_cases_exit_1_and_range_errors_exit_2() {
    let s = start();
    let dir = tmp();
    let o = run(
        &s,
        &["search", "home", "--category", "no-such-category"],
        dir.path(),
    );
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    let o = run(&s, &["show", "zzz:home"], dir.path());
    assert_eq!(o.status.code(), Some(1), "{}", stderr(&o));
    let o = run(
        &s,
        &["search", "home", "--start", "990", "-n", "20"],
        dir.path(),
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("at most 999"), "{}", stderr(&o));
    let o = run(
        &s,
        &["search", "home", "--per-set", "1000000000", "--json"],
        dir.path(),
    );
    assert_eq!(o.status.code(), Some(0), "{}", stderr(&o));
}
