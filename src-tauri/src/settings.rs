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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Animation {
    Full,
    Gentle,
    Still,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub corner: Corner,
    pub mascot: String,
    pub animation: Animation,
    /// mascot size in logical px
    pub size: u32,
    pub hidden: bool,
}

pub const MIN_SIZE: u32 = 64;
pub const MAX_SIZE: u32 = 192;

impl Default for Settings {
    fn default() -> Self {
        Self {
            corner: Corner::Br,
            mascot: "ember".into(),
            animation: Animation::Gentle,
            size: 128,
            hidden: false,
        }
    }
}

fn path(home: &Path) -> std::path::PathBuf {
    crate::token::mascot_dir(home).join("settings.json")
}

/// Missing or unreadable files fall back to defaults. Each field is read on its own,
/// so one bad value (e.g. from a newer version) never resets the others.
/// Size is clamped to the supported range.
pub fn load(home: &Path) -> Settings {
    let mut s = Settings::default();
    let obj = std::fs::read(path(home))
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    fn take<T: serde::de::DeserializeOwned>(obj: &serde_json::Map<String, serde_json::Value>, key: &str, into: &mut T) {
        if let Some(v) = obj.get(key).and_then(|v| serde_json::from_value(v.clone()).ok()) {
            *into = v;
        }
    }
    take(&obj, "corner", &mut s.corner);
    take(&obj, "mascot", &mut s.mascot);
    take(&obj, "animation", &mut s.animation);
    take(&obj, "size", &mut s.size);
    take(&obj, "hidden", &mut s.hidden);
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
        let s = Settings { corner: Corner::Tl, size: 96, hidden: true, ..Settings::default() };
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
    fn v1_file_loads_with_new_defaults() {
        let h = home("v1");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"),
            r#"{"corner":"tl","size":96,"autoHide":true,"hidden":false}"#).unwrap();
        let s = load(&h);
        assert_eq!(s.corner, Corner::Tl);
        assert_eq!(s.size, 96);
        assert_eq!(s.mascot, "ember");
        assert_eq!(s.animation, Animation::Gentle);
    }

    #[test]
    fn old_file_with_auto_hide_still_loads_other_fields() {
        let h = home("autohide");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"corner":"tl","autoHide":true}"#).unwrap();
        assert_eq!(load(&h), Settings { corner: Corner::Tl, ..Settings::default() });
    }

    #[test]
    fn mascot_and_animation_round_trip() {
        let h = home("v2");
        let s = Settings { mascot: "bolt".into(), animation: Animation::Still, ..Settings::default() };
        save(&h, &s).unwrap();
        assert_eq!(load(&h), s);
    }

    #[test]
    fn unknown_animation_keeps_other_fields() {
        let h = home("badanim");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"corner":"bl","animation":"turbo"}"#).unwrap();
        let s = load(&h);
        assert_eq!(s.animation, Animation::Gentle);
        assert_eq!(s.corner, Corner::Bl);
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
