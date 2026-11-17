//! Watches Claude Desktop (launch, focus, quit) and guesses when it is
//! generating a reply from its processes' CPU and network activity.

use crate::normalize::MascotEvent;

/// One activity sample summed over all Claude Desktop processes.
#[derive(Debug, Clone, Copy)]
pub struct ActivitySample {
    pub cpu_pct: f32,
    pub net_bytes_per_s: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityEdge {
    Start,
    Stop,
}

/// Tunables; calibrated against real replies in docs/desktop-calibration.md.
pub const CPU_ACTIVE_PCT: f32 = 6.0;
pub const NET_ACTIVE_BYTES_PER_S: u64 = 20_000;
pub const START_AFTER_MS: i64 = 1_500;
pub const STOP_AFTER_MS: i64 = 2_000;

/// Debounces samples into start/stop edges: activity must persist
/// `START_AFTER_MS` to start and quiet must persist `STOP_AFTER_MS` to stop.
#[derive(Debug, Default)]
pub struct Classifier {
    active: bool,
    /// when the current run of samples disagreeing with `active` began
    streak_since: Option<i64>,
}

impl Classifier {
    pub fn push(&mut self, s: ActivitySample, now_ms: i64) -> Option<ActivityEdge> {
        let busy = s.cpu_pct >= CPU_ACTIVE_PCT || s.net_bytes_per_s >= NET_ACTIVE_BYTES_PER_S;
        if busy == self.active {
            self.streak_since = None;
            return None;
        }
        let since = *self.streak_since.get_or_insert(now_ms);
        let needed = if busy { START_AFTER_MS } else { STOP_AFTER_MS };
        if now_ms - since < needed {
            return None;
        }
        self.active = busy;
        self.streak_since = None;
        Some(if busy { ActivityEdge::Start } else { ActivityEdge::Stop })
    }
}

const CLAUDE_BUNDLE_ID: &str = "com.anthropic.claudefordesktop";
const CLAUDE_APP_DIR: &str = "/Claude.app/Contents/";
const CLAUDE_MAIN_EXE: &str = "/Claude.app/Contents/MacOS/Claude";
const SAMPLE_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(target_os = "macos")]
fn claude_is_frontmost() -> bool {
    use objc2_app_kit::NSWorkspace;
    let ws = NSWorkspace::sharedWorkspace();
    ws.frontmostApplication()
        .and_then(|app| app.bundleIdentifier())
        .is_some_and(|id| id.to_string() == CLAUDE_BUNDLE_ID)
}

#[cfg(not(target_os = "macos"))]
fn claude_is_frontmost() -> bool {
    false
}

/// Samples Claude's network bytes per second into a shared value. `nettop`
/// block-buffers when piped, so it runs as repeated two-sample, one-second
/// runs (cumulative block, then a one-second delta block) that flush on exit.
fn spawn_net_sampler(latest: std::sync::Arc<std::sync::atomic::AtomicU64>) {
    use std::sync::atomic::Ordering;
    std::thread::spawn(move || loop {
        let out = std::process::Command::new("/usr/bin/nettop")
            .args(["-P", "-d", "-x", "-L", "2", "-s", "1", "-J", "bytes_in,bytes_out"])
            .stderr(std::process::Stdio::null())
            .output();
        let bytes = out.ok().and_then(|o| {
            let mut parser = NettopParser::default();
            let mut last = None;
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                last = parser.feed(line).or(last);
            }
            parser.finish().or(last)
        });
        match bytes {
            Some(b) => latest.store(b, Ordering::Relaxed),
            None => {
                latest.store(0, Ordering::Relaxed);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    });
}

/// Watches Claude Desktop and sends `source: "desktop"` events. If Claude is
/// already running when the mascot starts, no launch greeting is sent.
pub fn spawn(sink: std::sync::mpsc::Sender<MascotEvent>) {
    use std::sync::atomic::{AtomicU64, Ordering};
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

    let net = std::sync::Arc::new(AtomicU64::new(0));
    spawn_net_sampler(net.clone());

    std::thread::spawn(move || {
        let mut sys = System::new();
        let mut classifier = Classifier::default();
        let mut was_running: Option<bool> = None;
        let mut was_front = false;
        let send = |event: &'static str| sink.send(MascotEvent::new("desktop", event, now_ms())).is_ok();

        loop {
            sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_cpu().with_exe(sysinfo::UpdateKind::OnlyIfNotSet));
            let mut running = false;
            let mut cpu = 0.0f32;
            for p in sys.processes().values() {
                let Some(exe) = p.exe().and_then(|e| e.to_str()) else { continue };
                if exe.contains(CLAUDE_APP_DIR) {
                    cpu += p.cpu_usage();
                    running |= exe.ends_with(CLAUDE_MAIN_EXE);
                }
            }

            let ok = match (was_running, running) {
                (Some(false), true) => send("app_launch"),
                (Some(true), false) => {
                    classifier = Classifier::default();
                    send("app_quit")
                }
                _ => true,
            };
            if !ok {
                return; // the app is shutting down
            }
            was_running = Some(running);

            let front = running && claude_is_frontmost();
            if front && !was_front {
                send("app_focus");
            }
            was_front = front;

            if running {
                let sample = ActivitySample { cpu_pct: cpu, net_bytes_per_s: net.load(Ordering::Relaxed) };
                match classifier.push(sample, now_ms()) {
                    Some(ActivityEdge::Start) => { send("activity_start"); }
                    Some(ActivityEdge::Stop) => { send("activity_stop"); }
                    None => {}
                }
                if std::env::var_os("PUPPET_CALIBRATE").is_some() {
                    eprintln!("[calibrate] cpu={cpu:.1} net={}", sample.net_bytes_per_s);
                }
            }
            std::thread::sleep(SAMPLE_EVERY);
        }
    });
}

/// Parses `nettop -P -d -x -J bytes_in,bytes_out` output. Each block starts with
/// a `,bytes_in,bytes_out,` header; the first block is cumulative and skipped.
#[derive(Debug, Default)]
pub struct NettopParser {
    blocks_seen: u32,
    current: u64,
}

impl NettopParser {
    /// Returns the previous block's Claude bytes (in + out) when a new block starts.
    pub fn feed(&mut self, line: &str) -> Option<u64> {
        if line.starts_with(",bytes_in") {
            self.blocks_seen += 1;
            let done = std::mem::take(&mut self.current);
            return (self.blocks_seen >= 3).then_some(done);
        }
        let mut cols = line.split(',');
        let proc_name = cols.next()?.rsplit_once('.')?.0;
        if proc_name != "Claude" && !proc_name.starts_with("Claude Helper") {
            return None;
        }
        let bytes: u64 = cols.take(2).filter_map(|c| c.trim().parse::<u64>().ok()).sum();
        self.current += bytes;
        None
    }

    /// At end of output: the final block's bytes, if it was a delta block.
    pub fn finish(&mut self) -> Option<u64> {
        let last = std::mem::take(&mut self.current);
        (self.blocks_seen >= 2).then_some(last)
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn run(lines: &[&str]) -> Vec<u64> {
        let mut p = NettopParser::default();
        lines.iter().filter_map(|l| p.feed(l)).collect()
    }

    #[test]
    fn skips_cumulative_first_block_and_sums_claude_processes() {
        let out = run(&[
            ",bytes_in,bytes_out,",
            "Claude Helper.75749,569497,405267,",
            ",bytes_in,bytes_out,",
            "Claude Helper.75749,5755,4830,",
            "Safari.123,99999,99999,",
            ",bytes_in,bytes_out,",
            "Claude.75741,5203,2639,",
            "Claude Helper.75749,3607,1074,",
            ",bytes_in,bytes_out,",
        ]);
        assert_eq!(out, vec![5755 + 4830, 5203 + 2639 + 3607 + 1074]);
    }

    #[test]
    fn finish_returns_the_last_delta_block_of_a_two_sample_run() {
        let mut p = NettopParser::default();
        for l in [",bytes_in,bytes_out,", "Claude.1,900000,900000,", ",bytes_in,bytes_out,", "Claude.1,300,200,"] {
            assert_eq!(p.feed(l), None);
        }
        assert_eq!(p.finish(), Some(500));
    }

    #[test]
    fn finish_without_a_delta_block_is_none() {
        let mut p = NettopParser::default();
        p.feed(",bytes_in,bytes_out,");
        p.feed("Claude.1,900000,900000,");
        assert_eq!(p.finish(), None);
    }

    #[test]
    fn ignores_our_own_app_and_garbage_lines() {
        let out = run(&[
            ",bytes_in,bytes_out,", "x", ",bytes_in,bytes_out,",
            "Claude Puppet.1,100,100,", "Claude Helper.2,notanumber,5,", "",
            ",bytes_in,bytes_out,",
        ]);
        assert_eq!(out, vec![5]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUSY: ActivitySample = ActivitySample { cpu_pct: 20.0, net_bytes_per_s: 0 };
    const NET_BUSY: ActivitySample = ActivitySample { cpu_pct: 0.0, net_bytes_per_s: 50_000 };
    const QUIET: ActivitySample = ActivitySample { cpu_pct: 1.0, net_bytes_per_s: 5_000 };

    /// Feeds samples every 500 ms starting at `t0`, returning the edges with their times.
    fn feed(c: &mut Classifier, t0: i64, samples: &[ActivitySample]) -> Vec<(i64, ActivityEdge)> {
        samples
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let t = t0 + i as i64 * 500;
                c.push(*s, t).map(|e| (t, e))
            })
            .collect()
    }

    #[test]
    fn sustained_activity_starts_after_1_5_s() {
        let mut c = Classifier::default();
        assert_eq!(feed(&mut c, 0, &[BUSY; 4]), vec![(1500, ActivityEdge::Start)]);
    }

    #[test]
    fn network_alone_counts_as_activity() {
        let mut c = Classifier::default();
        assert_eq!(feed(&mut c, 0, &[NET_BUSY; 4]), vec![(1500, ActivityEdge::Start)]);
    }

    #[test]
    fn single_spike_is_ignored() {
        let mut c = Classifier::default();
        assert!(feed(&mut c, 0, &[BUSY, QUIET, QUIET, QUIET, QUIET]).is_empty());
    }

    #[test]
    fn short_gap_mid_reply_does_not_stop() {
        let mut c = Classifier::default();
        feed(&mut c, 0, &[BUSY; 4]);
        // 1.5 s of quiet (< 2 s), then busy again
        assert!(feed(&mut c, 2000, &[QUIET, QUIET, QUIET, BUSY, BUSY, BUSY]).is_empty());
    }

    #[test]
    fn two_seconds_of_quiet_stops() {
        let mut c = Classifier::default();
        feed(&mut c, 0, &[BUSY; 4]);
        assert_eq!(feed(&mut c, 2000, &[QUIET; 5]), vec![(4000, ActivityEdge::Stop)]);
    }

    #[test]
    fn background_chatter_stays_quiet() {
        let mut c = Classifier::default();
        assert!(feed(&mut c, 0, &[QUIET; 20]).is_empty());
    }
}
