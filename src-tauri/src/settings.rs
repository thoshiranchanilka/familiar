//! Mascot preferences, stored at `~/.claude-mascot/settings.json`.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Corner {
    Br,
    Bl,
    Tr,
    Tl,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub corner: Corner,
    /// mascot size in logical px
    pub size: u32,
    /// hide the mascot while it is asleep
    pub auto_hide: bool,
    pub hidden: bool,
}

pub const MIN_SIZE: u32 = 64;
pub const MAX_SIZE: u32 = 192;

impl Default for Settings {
    fn default() -> Self {
        Self { corner: Corner::Br, size: 128, auto_hide: false, hidden: false }
    }
}

fn path(home: &Path) -> std::path::PathBuf {
    crate::token::mascot_dir(home).join("settings.json")
}

/// Missing or unreadable files fall back to defaults; size is clamped to the supported range.
pub fn load(home: &Path) -> Settings {
    let mut s: Settings = std::fs::read(path(home))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    s.size = s.size.clamp(MIN_SIZE, MAX_SIZE);
    s
}

pub fn save(home: &Path, s: &Settings) -> std::io::Result<()> {
    let p = path(home);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(p, serde_json::to_vec_pretty(s)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn home(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("puppet-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn missing_file_gives_defaults() {
        assert_eq!(load(&home("missing")), Settings::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let h = home("roundtrip");
        let s = Settings { corner: Corner::Tl, size: 96, auto_hide: true, hidden: true };
        save(&h, &s).unwrap();
        assert_eq!(load(&h), s);
    }

    #[test]
    fn invalid_file_gives_defaults() {
        let h = home("invalid");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), "{oops").unwrap();
        assert_eq!(load(&h), Settings::default());
    }

    #[test]
    fn partial_file_fills_in_defaults() {
        let h = home("partial");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"corner":"bl"}"#).unwrap();
        assert_eq!(load(&h), Settings { corner: Corner::Bl, ..Settings::default() });
    }

    #[test]
    fn size_is_clamped() {
        let h = home("clamp");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"size":5000}"#).unwrap();
        assert_eq!(load(&h).size, MAX_SIZE);
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"size":1}"#).unwrap();
        assert_eq!(load(&h).size, MIN_SIZE);
    }
}
