//! Icon names in the `prefix:name` form Iconify uses.
//!
//! Prefixes and names are restricted to lowercase ASCII letters, digits and
//! hyphens. That is what Iconify icon sets use, and it means an id can go into
//! a URL path or a file name without escaping.

use std::fmt;

const MAX_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IconId {
    pub prefix: String,
    pub name: String,
}

impl IconId {
    /// Parses `prefix:name`.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        let (prefix, name) = s.split_once(':').ok_or_else(|| {
            format!("'{s}' is not an icon id: expected prefix:name, like mdi:home")
        })?;
        check_part(prefix, "prefix")?;
        check_part(name, "name")?;
        Ok(IconId {
            prefix: prefix.to_string(),
            name: name.to_string(),
        })
    }

    /// File stem used when saving the icon: `mdi-home`.
    pub fn file_stem(&self) -> String {
        format!("{}-{}", self.prefix, self.name)
    }
}

impl fmt::Display for IconId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.prefix, self.name)
    }
}

/// Checks an icon set prefix on its own (for `--set`).
pub fn check_prefix(prefix: &str) -> Result<(), String> {
    check_part(prefix, "prefix")
}

fn check_part(part: &str, what: &str) -> Result<(), String> {
    if part.is_empty() {
        return Err(format!("empty icon {what}"));
    }
    if part.len() > MAX_LEN {
        return Err(format!("icon {what} is longer than {MAX_LEN} characters"));
    }
    if part.starts_with('-') || part.ends_with('-') {
        return Err(format!("icon {what} '{part}' starts or ends with '-'"));
    }
    if let Some(c) = part
        .chars()
        .find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
    {
        return Err(format!(
            "icon {what} '{part}' contains {c:?}; only a-z, 0-9 and '-' are allowed"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats() {
        let id = IconId::parse(" material-symbols:home-outline ").unwrap();
        assert_eq!(id.prefix, "material-symbols");
        assert_eq!(id.name, "home-outline");
        assert_eq!(id.to_string(), "material-symbols:home-outline");
        assert_eq!(id.file_stem(), "material-symbols-home-outline");
    }

    #[test]
    fn rejects_unsafe_names() {
        for bad in [
            "home",
            ":home",
            "mdi:",
            "mdi:../etc",
            "mdi:Home",
            "mdi:ho me",
            "mdi:a/b",
            "-mdi:home",
            "mdi:home-",
            "mdi:a:b",
        ] {
            assert!(IconId::parse(bad).is_err(), "{bad} should be rejected");
        }
        assert!(IconId::parse(&format!("mdi:{}", "a".repeat(101))).is_err());
    }
}
