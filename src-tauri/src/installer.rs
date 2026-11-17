//! Connects Claude Code to the mascot by merging HTTP hooks and a status-line
//! command into `~/.claude/settings.json`, with a backup and a clean uninstall.

use crate::server::DEFAULT_PORT;
use crate::token::mascot_dir;
use serde_json::{json, Map, Value};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub const HOOK_EVENTS: [&str; 9] = [
    "SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "Stop",
    "StopFailure", "Notification", "SessionEnd", "PostModelSwitch",
];

#[derive(Debug, PartialEq)]
pub enum InstallError {
    InvalidSettings(String),
    Io(String),
}

#[derive(Debug, PartialEq)]
pub struct InstallReport {
    pub hooks_added: usize,
    pub chained_status_line: bool,
}

const STATUSLINE_SCRIPT: &str = include_str!("../../scripts/statusline.sh");
const HOOK_SCRIPT: &str = include_str!("../../scripts/hook.sh");
const STATUS_REFRESH_SECS: u64 = 30;

fn io(e: std::io::Error) -> InstallError {
    InstallError::Io(e.to_string())
}

fn settings_path(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}

fn hook_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/hook")
}

fn script_path(home: &Path) -> PathBuf {
    mascot_dir(home).join("statusline.sh")
}

/// Raw bytes of settings.json, or None when the file does not exist.
fn read_raw(home: &Path) -> Result<Option<Vec<u8>>, InstallError> {
    match std::fs::read(settings_path(home)) {
        Ok(b) => Ok(Some(b)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(e)),
    }
}

/// Parses settings bytes; blank files count as `{}`, anything but an object is an error.
fn parse(bytes: &[u8]) -> Result<Map<String, Value>, InstallError> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Map::new());
    }
    match serde_json::from_slice::<Value>(bytes) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(_) => Err(InstallError::InvalidSettings("settings.json is not a JSON object".into())),
        Err(e) => Err(InstallError::InvalidSettings(format!("settings.json is not valid JSON: {e}"))),
    }
}

/// Writes via a temp file and rename so a crash never leaves a half-written file.
/// Follows a symlinked settings.json (e.g. from a dotfiles repo) and keeps the file's permissions.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), InstallError> {
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let tmp = target.with_extension("json.puppet-tmp");
    std::fs::write(&tmp, bytes).map_err(io)?;
    if let Ok(meta) = std::fs::metadata(&target) {
        std::fs::set_permissions(&tmp, meta.permissions()).map_err(io)?;
    }
    std::fs::rename(&tmp, &target).map_err(io)
}

fn pretty(m: &Map<String, Value>) -> Vec<u8> {
    let mut s = serde_json::to_string_pretty(m).unwrap_or_else(|_| "{}".into());
    s.push('\n');
    s.into_bytes()
}

/// Ours: the current script hook, or an HTTP hook from an earlier version (migrated away).
fn is_our_hook(hook: &Value, url: &str) -> bool {
    hook["url"] == url
        || hook["command"].as_str().is_some_and(|c| c.ends_with("/.claude-mascot/hook.sh"))
}

fn is_our_group(group: &Value, url: &str) -> bool {
    group["hooks"].as_array().is_some_and(|hs| hs.iter().any(|h| is_our_hook(h, url)))
}

fn is_our_status_line(v: &Value) -> bool {
    v["command"].as_str().is_some_and(|c| c.ends_with(".claude-mascot/statusline.sh"))
}

pub fn install(home: &Path, token: &str, port: u16) -> Result<InstallReport, InstallError> {
    let raw = read_raw(home)?;
    let mut settings = parse(raw.as_deref().unwrap_or_default())?;
    let dir = mascot_dir(home);
    std::fs::create_dir_all(&dir).map_err(io)?;

    // Back up the pre-install file once; a re-install keeps the first backup.
    let backup = dir.join("settings.backup.json");
    let absent = dir.join("settings.absent");
    let has_our_status_line = settings.get("statusLine").is_some_and(is_our_status_line);
    if !is_installed(home) && !has_our_status_line {
        match &raw {
            Some(b) => {
                std::fs::write(&backup, b).map_err(io)?;
                let _ = std::fs::remove_file(&absent);
            }
            None => {
                std::fs::write(&absent, b"").map_err(io)?;
                let _ = std::fs::remove_file(&backup);
            }
        }
    }

    // Hooks: one HTTP group per event, updated in place on re-install.
    let url = hook_url(port);
    let hook_script = mascot_dir(home).join("hook.sh");
    std::fs::write(&hook_script, HOOK_SCRIPT).map_err(io)?;
    std::fs::set_permissions(&hook_script, std::fs::Permissions::from_mode(0o755)).map_err(io)?;
    let _ = token; // the script reads ~/.claude-mascot/token at run time; settings.json never holds it
    let our_hook = json!({ "type": "command", "command": hook_script.to_string_lossy(), "timeout": 5 });

    let hooks = settings.entry("hooks").or_insert_with(|| json!({}));
    if !hooks.is_object() {
        return Err(InstallError::InvalidSettings("\"hooks\" is not an object".into()));
    }
    let mut hooks_added = 0;
    for ev in HOOK_EVENTS {
        let list = hooks.as_object_mut().unwrap().entry(ev).or_insert_with(|| json!([]));
        let Some(arr) = list.as_array_mut() else {
            return Err(InstallError::InvalidSettings(format!("hooks.{ev} is not a list")));
        };
        // Remove every entry of ours (old HTTP or script), remembering the first group that had one,
        // then put exactly one script entry back there, leaving the user's hooks in that group alone.
        let mut home_group: Option<usize> = None;
        for (i, g) in arr.iter_mut().enumerate() {
            if let Some(hs) = g["hooks"].as_array_mut() {
                let before = hs.len();
                hs.retain(|h| !is_our_hook(h, &url));
                if hs.len() != before && home_group.is_none() {
                    home_group = Some(i);
                }
            }
        }
        match home_group {
            Some(i) => {
                if let Some(hs) = arr[i]["hooks"].as_array_mut() {
                    hs.insert(0, our_hook.clone());
                }
            }
            None => {
                arr.push(json!({ "hooks": [our_hook.clone()] }));
                hooks_added += 1;
            }
        }
        arr.retain(|g| !g["hooks"].as_array().is_some_and(Vec::is_empty));
    }

    // Status line: chain the user's previous command through our script.
    let mut chained_status_line = false;
    if let Some(prev) = settings.get("statusLine").filter(|v| !is_our_status_line(v)) {
        if let Some(cmd) = prev["command"].as_str() {
            std::fs::write(dir.join("prev-statusline"), cmd).map_err(io)?;
            chained_status_line = true;
        }
    }
    let script = script_path(home);
    std::fs::write(&script, STATUSLINE_SCRIPT).map_err(io)?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).map_err(io)?;
    settings.insert("statusLine".into(), json!({
        "type": "command",
        "command": script.to_string_lossy(),
        "refreshInterval": STATUS_REFRESH_SECS
    }));

    write_atomic(&settings_path(home), &pretty(&settings))?;
    Ok(InstallReport { hooks_added, chained_status_line })
}

pub fn uninstall(home: &Path) -> Result<(), InstallError> {
    let dir = mascot_dir(home);
    let backup_bytes = std::fs::read(dir.join("settings.backup.json")).ok();
    let was_absent = dir.join("settings.absent").exists();
    let original = parse(backup_bytes.as_deref().unwrap_or_default()).unwrap_or_default();

    if let Some(raw) = read_raw(home)? {
        let mut settings = parse(&raw)?;
        let url = hook_url(DEFAULT_PORT);

        // Remove our hook groups, then any lists or objects that became empty.
        if let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) {
            for list in hooks.values_mut() {
                if let Some(arr) = list.as_array_mut() {
                    for g in arr.iter_mut() {
                        if let Some(hs) = g["hooks"].as_array_mut() {
                            hs.retain(|h| !is_our_hook(h, &url));
                        }
                    }
                    arr.retain(|g| !g["hooks"].as_array().is_some_and(Vec::is_empty));
                }
            }
            hooks.retain(|_, v| !v.as_array().is_some_and(Vec::is_empty));
        }
        if settings.get("hooks").and_then(Value::as_object).is_some_and(Map::is_empty) {
            settings.remove("hooks");
        }
        if settings.get("statusLine").is_some_and(is_our_status_line) {
            let saved_cmd = std::fs::read_to_string(dir.join("prev-statusline")).ok().filter(|c| !c.is_empty());
            match (original.get("statusLine").filter(|v| !is_our_status_line(v)), saved_cmd) {
                (Some(prev), _) => settings.insert("statusLine".into(), prev.clone()),
                (None, Some(cmd)) => settings.insert("statusLine".into(), json!({ "type": "command", "command": cmd })),
                (None, None) => settings.remove("statusLine"),
            };
        }

        // Nothing else changed since install: put back the exact original bytes.
        let path = settings_path(home);
        if settings == original {
            match (&backup_bytes, was_absent) {
                (_, true) => std::fs::remove_file(&path).map_err(io)?,
                (Some(b), false) => write_atomic(&path, b)?,
                (None, false) => write_atomic(&path, &pretty(&settings))?,
            }
        } else {
            write_atomic(&path, &pretty(&settings))?;
        }
    }

    for f in ["settings.backup.json", "settings.absent", "prev-statusline", "statusline.sh", "hook.sh"] {
        let _ = std::fs::remove_file(dir.join(f));
    }
    Ok(())
}

/// True when settings.json still has HTTP hooks from an earlier version.
pub fn needs_migration(home: &Path) -> bool {
    let Ok(Some(raw)) = read_raw(home) else { return false };
    let Ok(settings) = parse(&raw) else { return false };
    let url = hook_url(DEFAULT_PORT);
    settings.get("hooks").and_then(Value::as_object).is_some_and(|h| {
        h.values()
            .filter_map(Value::as_array)
            .flatten()
            .filter_map(|g| g["hooks"].as_array())
            .flatten()
            .any(|x| x["url"] == url)
    })
}

pub fn is_installed(home: &Path) -> bool {
    let Ok(Some(raw)) = read_raw(home) else { return false };
    let Ok(settings) = parse(&raw) else { return false };
    let url = hook_url(DEFAULT_PORT);
    settings.get("hooks").and_then(Value::as_object).is_some_and(|h| {
        h.values().filter_map(Value::as_array).flatten().any(|g| is_our_group(g, &url))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::path::PathBuf;

    const PORT: u16 = 47653;

    fn home(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("puppet-inst-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join(".claude")).unwrap();
        d
    }
    fn settings_path(h: &Path) -> PathBuf { h.join(".claude/settings.json") }
    fn write_settings(h: &Path, s: &str) { std::fs::write(settings_path(h), s).unwrap(); }
    fn read_json(h: &Path) -> Value { serde_json::from_slice(&std::fs::read(settings_path(h)).unwrap()).unwrap() }
    fn is_ours(hook: &Value) -> bool {
        hook["command"].as_str().is_some_and(|c| c.ends_with(".claude-mascot/hook.sh"))
    }
    fn our_groups(v: &Value, ev: &str) -> usize {
        v["hooks"][ev].as_array().map_or(0, |a| a.iter().filter(|g| is_ours(&g["hooks"][0])).count())
    }

    #[test]
    fn fresh_install_creates_all_hooks_and_status_line() {
        let h = home("fresh");
        let r = install(&h, "tok", PORT).unwrap();
        assert_eq!(r.hooks_added, 9);
        let v = read_json(&h);
        for ev in HOOK_EVENTS {
            assert_eq!(our_groups(&v, ev), 1, "{ev}");
        }
        let hook = &v["hooks"]["Stop"][0]["hooks"][0];
        assert_eq!(hook["type"], "command");
        assert!(hook.get("url").is_none());
        let hook_script = h.join(".claude-mascot/hook.sh");
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&hook_script).unwrap().permissions().mode() & 0o111, 0o111);
        }
        assert_eq!(v["statusLine"]["type"], "command");
        assert!(v["statusLine"]["command"].as_str().unwrap().ends_with(".claude-mascot/statusline.sh"));
        assert!(is_installed(&h));
        let script = h.join(".claude-mascot/statusline.sh");
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(script).unwrap().permissions().mode() & 0o111, 0o111);
    }

    #[test]
    fn keeps_existing_hooks_and_settings() {
        let h = home("keep");
        write_settings(&h, r#"{"model":"opus","hooks":{"Stop":[{"hooks":[{"type":"command","command":"say done"}]}]}}"#);
        install(&h, "tok", PORT).unwrap();
        let v = read_json(&h);
        assert_eq!(v["model"], "opus");
        assert_eq!(v["hooks"]["Stop"][0]["hooks"][0]["command"], "say done");
        assert_eq!(our_groups(&v, "Stop"), 1);
    }

    #[test]
    fn installing_twice_does_not_duplicate_and_keeps_the_token_out_of_settings() {
        let h = home("twice");
        install(&h, "old", PORT).unwrap();
        let r = install(&h, "new", PORT).unwrap();
        assert_eq!(r.hooks_added, 0);
        let v = read_json(&h);
        assert_eq!(our_groups(&v, "PreToolUse"), 1);
        assert!(!std::fs::read_to_string(settings_path(&h)).unwrap().contains("Bearer"));
    }

    const OLD_HTTP_HOOK: &str = r#"{"type":"http","url":"http://127.0.0.1:47653/hook","timeout":2,"headers":{"Authorization":"Bearer x"}}"#;

    #[test]
    fn install_migrates_old_http_hooks() {
        let h = home("migrate");
        write_settings(&h, &format!(
            r#"{{"hooks":{{"Stop":[{{"hooks":[{OLD_HTTP_HOOK},{{"type":"command","command":"say done"}}]}}]}}}}"#));
        install(&h, "tok", PORT).unwrap();
        let text = std::fs::read_to_string(settings_path(&h)).unwrap();
        assert!(!text.contains("47653/hook"), "old HTTP hook left behind");
        let v = read_json(&h);
        assert!(has_user_hook(&v));
        let ours = v["hooks"]["Stop"].as_array().unwrap().iter()
            .flat_map(|g| g["hooks"].as_array().unwrap().iter())
            .filter(|x| is_ours(x)).count();
        assert_eq!(ours, 1);
    }

    #[test]
    fn detects_old_http_hooks_that_need_migrating() {
        let h = home("needs");
        assert!(!needs_migration(&h));
        write_settings(&h, &format!(r#"{{"hooks":{{"Stop":[{{"hooks":[{OLD_HTTP_HOOK}]}}]}}}}"#));
        assert!(needs_migration(&h));
        install(&h, "tok", PORT).unwrap();
        assert!(!needs_migration(&h));
    }

    #[test]
    fn uninstall_removes_old_http_hooks() {
        let h = home("migrate-un");
        write_settings(&h, &format!(r#"{{"hooks":{{"Stop":[{{"hooks":[{OLD_HTTP_HOOK}]}}]}}}}"#));
        uninstall(&h).unwrap();
        assert!(!std::fs::read_to_string(settings_path(&h)).unwrap().contains("47653/hook"));
    }

    fn run_hook(h: &Path, port: u16, input: &[u8]) -> (std::process::Output, std::time::Duration) {
        let start = std::time::Instant::now();
        let out = std::process::Command::new("sh")
            .arg(h.join(".claude-mascot/hook.sh"))
            .env("HOME", h)
            .env("CLAUDE_PUPPET_PORT", port.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(input)?;
                c.wait_with_output()
            })
            .unwrap();
        (out, start.elapsed())
    }

    #[test]
    fn hook_script_is_silent_and_quick_when_the_app_is_down() {
        let h = home("hook-down");
        install(&h, "tok", PORT).unwrap();
        std::fs::write(h.join(".claude-mascot/token"), "tok").unwrap();
        let big = format!(r#"{{"hook_event_name":"PostToolUse","session_id":"s1","tool_response":"{}"}}"#, "x".repeat(300_000));
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let (out, took) = run_hook(&h, free, big.as_bytes());
        assert!(out.status.success());
        assert!(out.stdout.is_empty(), "stdout: {}", String::from_utf8_lossy(&out.stdout));
        assert!(out.stderr.is_empty(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
        assert!(took < std::time::Duration::from_secs(2), "took {took:?}");
    }

    #[test]
    fn hook_script_sends_only_event_session_and_model() {
        use std::io::Read;
        let h = home("hook-send");
        install(&h, "tok", PORT).unwrap();
        std::fs::write(h.join(".claude-mascot/token"), "tok").unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let input = r#"{"session_id":"s-1","hook_event_name":"PostModelSwitch","to_model":"claude-opus-5-5","tool_response":"SECRET-FILE-CONTENT"}"#;
        let (out, _) = run_hook(&h, port, input.as_bytes());
        assert!(out.status.success());
        listener.set_nonblocking(false).unwrap();
        let (mut conn, _) = listener.accept().unwrap();
        conn.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        let mut req = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = conn.read(&mut buf) {
            if n == 0 { break; }
            req.extend_from_slice(&buf[..n]);
            if String::from_utf8_lossy(&req).contains("}") { break; }
        }
        let req = String::from_utf8_lossy(&req);
        assert!(req.starts_with("POST /hook "), "{req}");
        assert!(req.contains("Bearer tok"));
        assert!(req.contains(r#""hook_event_name":"PostModelSwitch""#), "{req}");
        assert!(req.contains(r#""session_id":"s-1""#));
        assert!(req.contains(r#""to_model":"claude-opus-5-5""#));
        assert!(!req.contains("SECRET"), "tool content leaked: {req}");
    }

    #[test]
    fn chains_previous_status_line() {
        let h = home("chain");
        write_settings(&h, r#"{"statusLine":{"type":"command","command":"echo hi"}}"#);
        let r = install(&h, "tok", PORT).unwrap();
        assert!(r.chained_status_line);
        assert_eq!(std::fs::read_to_string(h.join(".claude-mascot/prev-statusline")).unwrap(), "echo hi");
    }

    #[test]
    fn invalid_json_is_rejected_and_left_untouched() {
        let h = home("invalid");
        write_settings(&h, "{ not json");
        assert!(matches!(install(&h, "tok", PORT), Err(InstallError::InvalidSettings(_))));
        assert_eq!(std::fs::read_to_string(settings_path(&h)).unwrap(), "{ not json");
    }

    #[test]
    fn non_object_settings_are_rejected() {
        let h = home("array");
        write_settings(&h, "[1,2]");
        assert!(matches!(install(&h, "tok", PORT), Err(InstallError::InvalidSettings(_))));
    }

    #[test]
    fn empty_settings_file_is_treated_as_empty_object() {
        let h = home("empty");
        write_settings(&h, "");
        install(&h, "tok", PORT).unwrap();
        uninstall(&h).unwrap();
        assert_eq!(std::fs::read_to_string(settings_path(&h)).unwrap(), "");
    }

    #[test]
    fn uninstall_restores_original_bytes() {
        let h = home("restore");
        let original = "{\n    \"model\": \"opus\",\n    \"statusLine\": {\"type\":\"command\",\"command\":\"echo hi\"}\n}\n";
        write_settings(&h, original);
        install(&h, "tok", PORT).unwrap();
        assert!(h.join(".claude-mascot/settings.backup.json").exists());
        uninstall(&h).unwrap();
        assert_eq!(std::fs::read_to_string(settings_path(&h)).unwrap(), original);
        assert!(!is_installed(&h));
    }

    #[test]
    fn uninstall_removes_settings_file_that_did_not_exist() {
        let h = home("absent");
        install(&h, "tok", PORT).unwrap();
        uninstall(&h).unwrap();
        assert!(!settings_path(&h).exists());
    }

    #[test]
    fn uninstall_keeps_changes_made_after_install() {
        let h = home("later");
        write_settings(&h, r#"{"model":"opus"}"#);
        install(&h, "tok", PORT).unwrap();
        let mut v = read_json(&h);
        v["theme"] = json!("dark");
        write_settings(&h, &serde_json::to_string(&v).unwrap());
        uninstall(&h).unwrap();
        let after = read_json(&h);
        assert_eq!(after, json!({"model":"opus","theme":"dark"}));
    }

    fn add_user_hook_to_our_stop_group(h: &Path) {
        let mut v = read_json(h);
        let groups = v["hooks"]["Stop"].as_array_mut().unwrap();
        let ours = groups.iter_mut().find(|g| is_ours(&g["hooks"][0])).unwrap();
        ours["hooks"].as_array_mut().unwrap().push(json!({"type":"command","command":"say done"}));
        write_settings(h, &serde_json::to_string(&v).unwrap());
    }
    fn has_user_hook(v: &Value) -> bool {
        v["hooks"]["Stop"].as_array().is_some_and(|gs| gs.iter().any(|g| {
            g["hooks"].as_array().is_some_and(|hs| hs.iter().any(|x| x["command"] == "say done"))
        }))
    }

    #[test]
    fn reinstall_keeps_user_hooks_added_to_our_group() {
        let h = home("sharedgroup-re");
        install(&h, "tok", PORT).unwrap();
        add_user_hook_to_our_stop_group(&h);
        install(&h, "tok2", PORT).unwrap();
        let v = read_json(&h);
        assert!(has_user_hook(&v));
        assert_eq!(our_groups(&v, "Stop"), 1);
    }

    #[test]
    fn uninstall_keeps_user_hooks_added_to_our_group() {
        let h = home("sharedgroup-un");
        install(&h, "tok", PORT).unwrap();
        add_user_hook_to_our_stop_group(&h);
        uninstall(&h).unwrap();
        let v = read_json(&h);
        assert!(has_user_hook(&v));
        assert!(!is_installed(&h));
    }

    #[test]
    fn uninstall_without_backup_restores_previous_status_line() {
        let h = home("nobackup");
        write_settings(&h, r#"{"statusLine":{"type":"command","command":"echo hi"}}"#);
        install(&h, "tok", PORT).unwrap();
        std::fs::remove_file(h.join(".claude-mascot/settings.backup.json")).unwrap();
        uninstall(&h).unwrap();
        assert_eq!(read_json(&h)["statusLine"]["command"], "echo hi");
    }

    #[test]
    fn reconnect_after_hooks_were_removed_keeps_the_real_backup() {
        let h = home("stale");
        write_settings(&h, r#"{"statusLine":{"type":"command","command":"echo hi"}}"#);
        install(&h, "tok", PORT).unwrap();
        let mut v = read_json(&h);
        v.as_object_mut().unwrap().remove("hooks");
        write_settings(&h, &serde_json::to_string(&v).unwrap());
        install(&h, "tok", PORT).unwrap();
        uninstall(&h).unwrap();
        assert_eq!(read_json(&h)["statusLine"]["command"], "echo hi");
    }

    #[test]
    fn writes_through_a_symlink_and_keeps_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let h = home("symlink");
        let real = h.join("dotfiles-settings.json");
        std::fs::write(&real, r#"{"model":"opus"}"#).unwrap();
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::os::unix::fs::symlink(&real, settings_path(&h)).unwrap();
        install(&h, "tok", PORT).unwrap();
        assert!(std::fs::symlink_metadata(settings_path(&h)).unwrap().file_type().is_symlink());
        assert!(std::fs::read_to_string(&real).unwrap().contains("statusLine"));
        assert_eq!(std::fs::metadata(&real).unwrap().permissions().mode() & 0o777, 0o600);
        uninstall(&h).unwrap();
        assert!(std::fs::symlink_metadata(settings_path(&h)).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_to_string(&real).unwrap(), r#"{"model":"opus"}"#);
    }

    #[test]
    fn status_line_script_prints_the_model_when_there_is_no_previous_command() {
        let h = home("script-own");
        install(&h, "tok", PORT).unwrap();
        let out = run_script(&h, br#"{"model":{"id":"claude-x","display_name":"Opus 5.5"},"cwd":"/p"}"#);
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "🔥 Ember · Opus 5.5");
    }

    fn run_script(h: &Path, input: &[u8]) -> std::process::Output {
        std::process::Command::new("sh")
            .arg(h.join(".claude-mascot/statusline.sh"))
            .env("HOME", h)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(input)?;
                c.wait_with_output()
            })
            .unwrap()
    }

    #[test]
    fn status_line_script_never_fails_and_runs_previous_command() {
        let h = home("script");
        install(&h, "tok", PORT).unwrap();
        std::fs::write(h.join(".claude-mascot/prev-statusline"), "cat >/dev/null; echo prev-output").unwrap();
        let out = std::process::Command::new("sh")
            .arg(h.join(".claude-mascot/statusline.sh"))
            .env("HOME", &h)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(br#"{"model":{"display_name":"X"}}"#)?;
                c.wait_with_output()
            })
            .unwrap();
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "prev-output");
    }
}
