//! Coarse license classes for icon sets, keyed by SPDX identifier.
//!
//! The class is a reading aid that answers "what do I have to do to use this
//! icon", not legal advice. Anything not in the table is `Unknown`, so a new or
//! unusual license is never shown as safe by accident.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Class {
    /// CC0, Unlicense: no conditions.
    PublicDomain,
    /// MIT, Apache-2.0, ISC, BSD, OFL: keep the license notice with the files.
    Permissive,
    /// CC BY: credit the author where the icon is shown.
    Attribution,
    /// GPL, LGPL, MPL, CC BY-SA: share-alike or copyleft terms.
    Copyleft,
    /// CC BY-NC and friends: not for commercial use.
    NonCommercial,
    /// No SPDX identifier, or one this table does not know.
    Unknown,
}

impl Class {
    pub const ALL: [Class; 6] = [
        Class::PublicDomain,
        Class::Permissive,
        Class::Attribution,
        Class::Copyleft,
        Class::NonCommercial,
        Class::Unknown,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Class::PublicDomain => "public-domain",
            Class::Permissive => "permissive",
            Class::Attribution => "attribution",
            Class::Copyleft => "copyleft",
            Class::NonCommercial => "non-commercial",
            Class::Unknown => "unknown",
        }
    }

    pub fn note(self) -> &'static str {
        match self {
            Class::PublicDomain => "No conditions.",
            Class::Permissive => "Keep the license notice when you redistribute the icon files.",
            Class::Attribution => "Credit the author where the icon is used.",
            Class::Copyleft => "Share-alike or copyleft terms: read the license before shipping.",
            Class::NonCommercial => "Not for commercial use.",
            Class::Unknown => "No recognized SPDX identifier: read the license.",
        }
    }

    /// True for classes that ask for nothing beyond keeping a notice.
    pub fn no_attribution(self) -> bool {
        matches!(self, Class::PublicDomain | Class::Permissive)
    }

    pub fn parse(s: &str) -> Result<Class, String> {
        let s = s.trim().to_ascii_lowercase();
        let found = match s.as_str() {
            "pd" | "public-domain" => Some(Class::PublicDomain),
            "noncommercial" | "nc" => Some(Class::NonCommercial),
            _ => Class::ALL.into_iter().find(|c| c.label() == s),
        };
        found.ok_or_else(|| {
            let names: Vec<&str> = Class::ALL.iter().map(|c| c.label()).collect();
            format!(
                "unknown license class '{s}' (expected {})",
                names.join(", ")
            )
        })
    }
}

pub fn classify(spdx: Option<&str>) -> Class {
    let Some(id) = spdx.map(str::trim).filter(|s| !s.is_empty()) else {
        return Class::Unknown;
    };
    // SPDX expressions (`MIT OR GPL-3.0-only`, `Apache-2.0 WITH LLVM-exception`)
    // combine terms; this table only reads single identifiers.
    if id.contains(|c: char| c.is_whitespace() || c == '(' || c == ')') {
        return Class::Unknown;
    }
    let upper = id.to_ascii_uppercase();
    if upper.starts_with("CC-BY") {
        if upper.contains("-NC") {
            return Class::NonCommercial;
        }
        if upper.contains("-SA") {
            return Class::Copyleft;
        }
        if upper.contains("-ND") {
            // No-derivatives licenses forbid recoloring or resizing edits.
            return Class::Unknown;
        }
        return Class::Attribution;
    }
    match upper.as_str() {
        "CC0-1.0" | "UNLICENSE" => Class::PublicDomain,
        "MIT" | "MIT-0" | "ISC" | "APACHE-2.0" | "BSD-2-CLAUSE" | "BSD-3-CLAUSE" | "OFL-1.1"
        | "ZLIB" => Class::Permissive,
        "MPL-2.0" => Class::Copyleft,
        _ if upper.starts_with("GPL-")
            || upper.starts_with("LGPL-")
            || upper.starts_with("AGPL-") =>
        {
            Class::Copyleft
        }
        _ => Class::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_the_licenses_iconify_uses() {
        // Every SPDX id present in api.iconify.design/collections on 2026-11-02.
        let table = [
            ("MIT", Class::Permissive),
            ("CC-BY-4.0", Class::Attribution),
            ("Apache-2.0", Class::Permissive),
            ("OFL-1.1", Class::Permissive),
            ("CC0-1.0", Class::PublicDomain),
            ("CC-BY-SA-4.0", Class::Copyleft),
            ("ISC", Class::Permissive),
            ("GPL-2.0-or-later", Class::Copyleft),
            ("CC-BY-SA-3.0", Class::Copyleft),
            ("GPL-3.0-or-later", Class::Copyleft),
            ("MPL-2.0", Class::Copyleft),
            ("GPL-2.0-only", Class::Copyleft),
            ("BSD-3-Clause", Class::Permissive),
            ("Unlicense", Class::PublicDomain),
            ("CC-BY-NC-SA-4.0", Class::NonCommercial),
            ("CC-BY-3.0", Class::Attribution),
            ("GPL-3.0", Class::Copyleft),
            ("CC-BY-NC-4.0", Class::NonCommercial),
        ];
        for (spdx, class) in table {
            assert_eq!(classify(Some(spdx)), class, "{spdx}");
        }
    }

    #[test]
    fn unknown_is_never_safe() {
        assert_eq!(classify(None), Class::Unknown);
        assert_eq!(classify(Some("")), Class::Unknown);
        assert_eq!(classify(Some("LicenseRef-custom")), Class::Unknown);
        assert_eq!(classify(Some("CC-BY-ND-4.0")), Class::Unknown);
        assert_eq!(classify(Some("CC-BY-4.0 AND GPL-3.0-only")), Class::Unknown);
        assert_eq!(classify(Some("(MIT OR Apache-2.0)")), Class::Unknown);
        assert!(!Class::Unknown.no_attribution());
    }

    #[test]
    fn parses_class_names() {
        assert_eq!(Class::parse("pd").unwrap(), Class::PublicDomain);
        assert_eq!(
            Class::parse("Non-Commercial").unwrap(),
            Class::NonCommercial
        );
        assert_eq!(Class::parse("copyleft").unwrap(), Class::Copyleft);
        assert!(Class::parse("free").is_err());
    }
}
