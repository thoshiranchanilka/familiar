//! Watches Claude Desktop (launch, focus, quit) and guesses when it is
//! generating a reply from its processes' CPU use. (Network sampling via
//! `nettop` was removed: it cost over a full CPU core continuously.)

use crate::normalize::MascotEvent;

/// One activity sample summed over all Claude Desktop processes.
#[derive(Debug, Clone, Copy)]
pub struct ActivitySample {
    pub cpu_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityEdge {
    Start,
    Stop,
}

/// Tunables; calibrated against real replies in docs/desktop-calibration.md.
pub const CPU_ACTIVE_PCT: f32 = 6.0;
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
        let busy = s.cpu_pct >= CPU_ACTIVE_PCT;
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
const CLOSED_CHECK_EVERY: std::time::Duration = std::time::Duration::from_secs(2);
const RUNNING_SAMPLE_EVERY: std::time::Duration = std::time::Duration::from_secs(1);
/// While Claude runs, rescan every process this often to pick up new helper processes.
const HELPER_RESCAN_SECS: u64 = 30;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// (running, frontmost) for Claude Desktop, straight from AppKit: no process-list scan.
#[cfg(target_os = "macos")]
fn claude_app_state() -> (bool, bool) {
    use objc2_app_kit::{NSRunningApplication, NSWorkspace};
    use objc2_foundation::NSString;
    objc2::rc::autoreleasepool(|_| {
        let id = NSString::from_str(CLAUDE_BUNDLE_ID);
        let running = NSRunningApplication::runningApplicationsWithBundleIdentifier(&id).count() > 0;
        let front = running
            && NSWorkspace::sharedWorkspace()
                .frontmostApplication()
                .and_then(|app| app.bundleIdentifier())
                .is_some_and(|b| b.to_string() == CLAUDE_BUNDLE_ID);
        (running, front)
    })
}

#[cfg(not(target_os = "macos"))]
fn claude_app_state() -> (bool, bool) {
    (false, false)
}

/// Whether Claude Desktop is running right now.
pub fn claude_running() -> bool {
    claude_app_state().0
}

/// Watches Claude Desktop and sends `source: "desktop"` events. If Claude is
/// already running when the mascot starts, no launch greeting is sent.
/// While Claude is closed this costs one AppKit query every 2 s; while it runs,
/// only Claude's own processes are sampled, once a second.
pub fn spawn(sink: std::sync::mpsc::Sender<MascotEvent>) {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

    std::thread::spawn(move || {
        let mut sys = System::new();
        let mut classifier = Classifier::default();
        let mut was_running: Option<bool> = None;
        let mut was_front = false;
        let mut known: Vec<Pid> = Vec::new();
        let mut last_full_scan: Option<std::time::Instant> = None;
        let refresh = ProcessRefreshKind::nothing().with_cpu().with_exe(sysinfo::UpdateKind::OnlyIfNotSet);
        let send = |event: &'static str| sink.send(MascotEvent::new("desktop", event, now_ms())).is_ok();

        loop {
            let (running, front) = claude_app_state();

            let ok = match (was_running, running) {
                (Some(false), true) => send("app_launch"),
                (Some(true), false) => {
                    classifier = Classifier::default();
                    known.clear();
                    last_full_scan = None;
                    send("app_quit")
                }
                _ => true,
            };
            if !ok {
                return; // the app is shutting down
            }
            was_running = Some(running);

            if front && !was_front {
                send("app_focus");
            }
            was_front = front;

            let since_scan = last_full_scan.map_or(u64::MAX, |t| t.elapsed().as_secs());
            let plan = plan_tick(running, since_scan);
            if running {
                if plan.full_scan || known.is_empty() {
                    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh);
                    last_full_scan = Some(std::time::Instant::now());
                } else {
                    sys.refresh_processes_specifics(ProcessesToUpdate::Some(&known), true, refresh);
                }
                let scan = scan_claude(sys.processes().iter().filter_map(|(pid, p)| {
                    Some((pid.as_u32(), p.exe()?.to_str()?, p.cpu_usage()))
                }));
                known = scan.pids.iter().map(|p| Pid::from_u32(*p)).collect();
                match classifier.push(ActivitySample { cpu_pct: scan.cpu }, now_ms()) {
                    Some(ActivityEdge::Start) => { send("activity_start"); }
                    Some(ActivityEdge::Stop) => { send("activity_stop"); }
                    None => {}
                }
                if std::env::var_os("PUPPET_CALIBRATE").is_some() {
                    eprintln!("[calibrate] cpu={:.1}", scan.cpu);
                }
            }
            std::thread::sleep(plan.delay);
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickPlan {
    pub delay: std::time::Duration,
    pub full_scan: bool,
}

/// How long to sleep after this tick and whether to rescan every process for Claude's helpers.
pub fn plan_tick(running: bool, secs_since_full_scan: u64) -> TickPlan {
    if running {
        TickPlan { delay: RUNNING_SAMPLE_EVERY, full_scan: secs_since_full_scan >= HELPER_RESCAN_SECS }
    } else {
        TickPlan { delay: CLOSED_CHECK_EVERY, full_scan: false }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct ClaudeScan {
    pub running: bool,
    pub pids: Vec<u32>,
    pub cpu: f32,
}

/// Picks Claude Desktop's processes out of `(pid, exe path, cpu %)` tuples.
pub fn scan_claude<'a>(procs: impl Iterator<Item = (u32, &'a str, f32)>) -> ClaudeScan {
    let mut s = ClaudeScan::default();
    for (pid, exe, cpu) in procs {
        if exe.contains(CLAUDE_APP_DIR) {
            s.pids.push(pid);
            s.cpu += cpu;
            s.running |= exe.ends_with(CLAUDE_MAIN_EXE);
        }
    }
    s.pids.sort_unstable();
    s
}

#[cfg(test)]
mod plan_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn closed_claude_is_checked_every_2_s_without_scanning() {
        assert_eq!(plan_tick(false, 0), TickPlan { delay: Duration::from_secs(2), full_scan: false });
        assert_eq!(plan_tick(false, 999), TickPlan { delay: Duration::from_secs(2), full_scan: false });
    }

    #[test]
    fn running_claude_is_sampled_every_second() {
        assert_eq!(plan_tick(true, 0), TickPlan { delay: Duration::from_secs(1), full_scan: false });
        assert_eq!(plan_tick(true, 29).full_scan, false);
    }

    #[test]
    fn running_claude_rescans_for_new_helpers_every_30_s() {
        assert_eq!(plan_tick(true, 30), TickPlan { delay: Duration::from_secs(1), full_scan: true });
    }
}

#[cfg(test)]
mod scan_tests {
    use super::*;

    #[test]
    fn finds_claude_processes_and_sums_their_cpu() {
        let procs = [
            (1, "/Applications/Claude.app/Contents/MacOS/Claude", 2.0),
            (2, "/Applications/Claude.app/Contents/Frameworks/Claude Helper (Renderer).app/Contents/MacOS/Claude Helper (Renderer)", 3.5),
            (3, "/Applications/Claude Puppet.app/Contents/MacOS/claude-puppet", 50.0),
            (4, "/Applications/Safari.app/Contents/MacOS/Safari", 20.0),
        ];
        let s = scan_claude(procs.iter().map(|(p, e, c)| (*p, *e, *c)));
        assert!(s.running);
        assert_eq!(s.pids, vec![1, 2]);
        assert_eq!(s.cpu, 5.5);
    }

    #[test]
    fn helpers_without_the_main_app_do_not_count_as_running() {
        let procs = [(2, "/Applications/Claude.app/Contents/Frameworks/Claude Helper.app/Contents/MacOS/Claude Helper", 1.0)];
        assert!(!scan_claude(procs.iter().map(|(p, e, c)| (*p, *e, *c))).running);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUSY: ActivitySample = ActivitySample { cpu_pct: 20.0 };
    const QUIET: ActivitySample = ActivitySample { cpu_pct: 1.0 };

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
