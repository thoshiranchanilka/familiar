// Whether the mascot is on screen. It appears only while Claude Code is in use, except when it
// has something to say: it isn't connected yet, the app hit a problem, or a notice is up.
// Settings being open also shows it, so changes can be previewed. The user's Hide always wins.
export interface VisibilityFacts {
  hidden: boolean;
  connected: boolean;
  present: boolean;
  settingsOpen: boolean;
  noticeActive: boolean;
  serverError: boolean;
}

export function shouldShow(v: VisibilityFacts): boolean {
  return !v.hidden && (!v.connected || v.serverError || v.present || v.settingsOpen || v.noticeActive);
}
