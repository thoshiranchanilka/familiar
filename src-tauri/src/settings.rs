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
    /// Extra distance (logical px) from the corner's side edge and top/bottom edge,
    /// set by the Advanced position sliders or by dragging the mascot.
    pub offset_x: u32,
    pub offset_y: u32,
    /// Set once Open at login has been turned on for the first time, so it only happens once.
    pub login_set_up: bool,
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
            offset_x: 0,
            offset_y: 0,
            login_set_up: false,
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
    take(&obj, "loginSetUp", &mut s.login_set_up);
    take(&obj, "offsetX", &mut s.offset_x);
    take(&obj, "offsetY", &mut s.offset_y);
    s.size = s.size.clamp(MIN_SIZE, MAX_SIZE);
    s
}

#[derive(Debug, PartialEq)]
pub enum LoginPlan {
    /// Running from the disk image or a translocated copy that macOS deletes later:
    /// a login item would point at a path that disappears, so wait for a proper install.
    Skip,
    /// Turn Open at login on; record `login_set_up` only if that succeeds.
    EnableFirstTime,
    /// Already set up once: leave the user's choice alone, but if it is on, re-register
    /// so the login item follows the app if it was moved.
    RefreshIfEnabled,
}

pub fn login_plan(login_set_up: bool, exe: &str) -> LoginPlan {
    if exe.starts_with("/Volumes/") || exe.contains("/AppTranslocation/") {
        LoginPlan::Skip
    } else if login_set_up {
        LoginPlan::RefreshIfEnabled
    } else {
        LoginPlan::EnableFirstTime
    }
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
    fn login_is_left_alone_when_running_from_the_dmg_or_a_translocated_copy() {
        assert_eq!(login_plan(false, "/Volumes/Familiar/Familiar.app/Contents/MacOS/familiar"), LoginPlan::Skip);
        assert_eq!(login_plan(true, "/private/var/folders/x/AppTranslocation/ABC/d/Familiar.app/Contents/MacOS/familiar"), LoginPlan::Skip);
    }

    #[test]
    fn login_turns_on_first_time_then_refreshes_the_path() {
        let exe = "/Applications/Familiar.app/Contents/MacOS/familiar";
        assert_eq!(login_plan(false, exe), LoginPlan::EnableFirstTime);
        assert_eq!(login_plan(true, exe), LoginPlan::RefreshIfEnabled);
    }


    #[test]
    fn existing_file_without_marker_counts_as_first_run() {
        let h = home("nomarker");
        std::fs::create_dir_all(h.join(".claude-mascot")).unwrap();
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"corner":"bl","mascot":"bolt"}"#).unwrap();
        let s = load(&h);
        assert!(!s.login_set_up);
        assert_eq!(s.mascot, "bolt");
    }

    #[test]
    fn marker_survives_save_and_load() {
        let h = home("marker");
        let s = Settings { login_set_up: true, ..Settings::default() };
        save(&h, &s).unwrap();
        assert!(load(&h).login_set_up);
    }

    #[test]
    fn offsets_round_trip_and_default_to_the_corner() {
        let h = home("offsets");
        assert_eq!((load(&h).offset_x, load(&h).offset_y), (0, 0));
        let s = Settings { offset_x: 140, offset_y: 60, ..Settings::default() };
        save(&h, &s).unwrap();
        assert_eq!(load(&h), s);
        std::fs::write(h.join(".claude-mascot/settings.json"), r#"{"offsetX":-5,"offsetY":"x","corner":"tl"}"#).unwrap();
        let s = load(&h);
        assert_eq!((s.offset_x, s.offset_y, s.corner), (0, 0, Corner::Tl));
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
