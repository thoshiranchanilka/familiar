//! Turns raw Claude Code hook and status-line JSON into `MascotEvent`s.
//! Uses loose `Value` lookups so unknown or missing fields from other
//! Claude Code versions never break parsing.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_hour_pct: Option<f64>,
    /// epoch ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_hour_resets_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_pct: Option<f64>,
    /// epoch ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seven_day_resets_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_pct: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MascotEvent {
    pub source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub event: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// epoch ms
    pub ts: i64,
}

impl MascotEvent {
    pub fn new(source: &'static str, event: &'static str, ts: i64) -> Self {
        Self { source, session_id: None, event, model: None, usage: None, ts }
    }
}

fn str_at(raw: &Value, ptr: &str) -> Option<String> {
    raw.pointer(ptr).and_then(Value::as_str).map(str::to_owned)
}

fn num_at(raw: &Value, ptr: &str) -> Option<f64> {
    raw.pointer(ptr).and_then(Value::as_f64)
}

/// Unix seconds in the payload become epoch ms for the UI.
fn secs_to_ms(raw: &Value, ptr: &str) -> Option<i64> {
    num_at(raw, ptr).map(|s| (s * 1000.0) as i64)
}

pub fn from_hook(raw: &Value, now_ms: i64) -> Option<MascotEvent> {
    let event = match raw.get("hook_event_name")?.as_str()? {
        "SessionStart" => "session_start",
        "UserPromptSubmit" => "prompt",
        "PreToolUse" | "PostToolUse" => "tool",
        "Stop" => "stop",
        "StopFailure" => "error",
        "Notification" => "attention",
        "SessionEnd" => "session_end",
        "PostModelSwitch" => "status",
        _ => return None,
    };
    let mut e = MascotEvent::new("code", event, now_ms);
    e.session_id = str_at(raw, "/session_id");
    e.model = str_at(raw, "/to_model").or_else(|| str_at(raw, "/model"));
    Some(e)
}

pub fn from_status(raw: &Value, now_ms: i64) -> Option<MascotEvent> {
    let mut e = MascotEvent::new("code", "status", now_ms);
    e.session_id = str_at(raw, "/session_id");
    e.model = str_at(raw, "/model/display_name");
    e.usage = Some(Usage {
        five_hour_pct: num_at(raw, "/rate_limits/five_hour/used_percentage"),
        five_hour_resets_at: secs_to_ms(raw, "/rate_limits/five_hour/resets_at"),
        seven_day_pct: num_at(raw, "/rate_limits/seven_day/used_percentage"),
        seven_day_resets_at: secs_to_ms(raw, "/rate_limits/seven_day/resets_at"),
        context_pct: num_at(raw, "/context_window/used_percentage"),
    });
    Some(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hook(name: &str) -> Value {
        json!({ "session_id": "s1", "transcript_path": "/x.jsonl", "cwd": "/p", "hook_event_name": name })
    }

    #[test]
    fn maps_each_hook_event() {
        let cases = [
            ("SessionStart", "session_start"),
            ("UserPromptSubmit", "prompt"),
            ("PreToolUse", "tool"),
            ("PostToolUse", "tool"),
            ("Stop", "stop"),
            ("StopFailure", "error"),
            ("Notification", "attention"),
            ("SessionEnd", "session_end"),
        ];
        for (raw, want) in cases {
            let e = from_hook(&hook(raw), 42).unwrap_or_else(|| panic!("{raw} gave None"));
            assert_eq!(e.event, want, "{raw}");
            assert_eq!(e.source, "code");
            assert_eq!(e.session_id.as_deref(), Some("s1"));
            assert_eq!(e.ts, 42);
        }
    }

    #[test]
    fn model_switch_becomes_status_with_model() {
        let raw = json!({ "session_id": "s1", "hook_event_name": "PostModelSwitch", "from_model": "a", "to_model": "claude-opus-5-5" });
        let e = from_hook(&raw, 1).unwrap();
        assert_eq!(e.event, "status");
        assert_eq!(e.model.as_deref(), Some("claude-opus-5-5"));
    }

    #[test]
    fn session_start_carries_model_when_present() {
        let mut raw = hook("SessionStart");
        raw["model"] = json!("claude-sonnet-5-5");
        assert_eq!(from_hook(&raw, 1).unwrap().model.as_deref(), Some("claude-sonnet-5-5"));
    }

    #[test]
    fn unknown_hook_is_ignored() {
        assert_eq!(from_hook(&hook("CwdChanged"), 1), None);
        assert_eq!(from_hook(&json!({}), 1), None);
    }

    #[test]
    fn missing_session_id_still_returns_event() {
        let e = from_hook(&json!({ "hook_event_name": "Stop" }), 1).unwrap();
        assert_eq!(e.session_id, None);
        assert_eq!(e.event, "stop");
    }

    #[test]
    fn full_status_line_payload() {
        let raw = json!({
            "session_id": "s1",
            "model": { "id": "claude-opus-5-5", "display_name": "Opus 5.5" },
            "cost": { "total_cost_usd": 0.01 },
            "context_window": { "used_percentage": 8, "context_window_size": 200000 },
            "rate_limits": {
                "five_hour": { "used_percentage": 23.5, "resets_at": 1_800_000_000 },
                "seven_day": { "used_percentage": 41.2, "resets_at": 1_800_100_000 }
            },
            "some_future_field": true
        });
        let e = from_status(&raw, 5).unwrap();
        assert_eq!(e.event, "status");
        assert_eq!(e.model.as_deref(), Some("Opus 5.5"));
        assert_eq!(e.session_id.as_deref(), Some("s1"));
        assert_eq!(e.usage.unwrap(), Usage {
            five_hour_pct: Some(23.5),
            five_hour_resets_at: Some(1_800_000_000_000),
            seven_day_pct: Some(41.2),
            seven_day_resets_at: Some(1_800_100_000_000),
            context_pct: Some(8.0),
        });
    }

    #[test]
    fn status_without_rate_limits_keeps_context_only() {
        let raw = json!({ "model": { "display_name": "Sonnet 5.5" }, "context_window": { "used_percentage": 12.5 } });
        assert_eq!(from_status(&raw, 1).unwrap().usage.unwrap(), Usage { context_pct: Some(12.5), ..Default::default() });
    }

    #[test]
    fn null_percentages_are_omitted() {
        let raw = json!({
            "context_window": { "used_percentage": null },
            "rate_limits": { "five_hour": { "used_percentage": null, "resets_at": 1 } }
        });
        let u = from_status(&raw, 1).unwrap().usage.unwrap();
        assert_eq!(u.five_hour_pct, None);
        assert_eq!(u.context_pct, None);
    }

    #[test]
    fn serializes_camel_case_without_nulls() {
        let mut e = MascotEvent::new("code", "status", 7);
        e.usage = Some(Usage { five_hour_pct: Some(1.0), ..Default::default() });
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(s, r#"{"source":"code","event":"status","usage":{"fiveHourPct":1.0},"ts":7}"#);
    }
}
