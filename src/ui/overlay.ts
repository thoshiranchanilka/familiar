// Overlay entry point: listens for mascot events, runs the mood reducer, renders the mascot.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { initialState, reduce } from "../mood/reducer";
import type { MascotEvent, MascotState, Mood } from "../mood/types";

const stage = document.getElementById("stage")!;
const style = document.createElement("style");
style.textContent = `
  #stage { position: fixed; right: 12px; bottom: 12px; display: grid; justify-items: end; gap: 4px;
           font: 12px -apple-system, system-ui, sans-serif; }
  .placeholder { font-size: 128px; line-height: 1; animation: bob 1.6s ease-in-out infinite; }
  .mood { background: rgba(20,20,24,.75); color: #fff; padding: 2px 8px; border-radius: 8px; }
  .mood:empty { display: none; }
  @keyframes bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-6px); } }
  @media (prefers-reduced-motion: reduce) { .placeholder { animation: none; } }
`;
document.head.appendChild(style);

const label = document.createElement("div");
label.className = "mood";
const mascot = document.createElement("div");
mascot.className = "placeholder";
mascot.textContent = "🔥";
stage.append(label, mascot);

// Placeholder rendering until Ember lands in Task 8.
const LABELS: Record<Mood, string> = {
  greeting: "Hi!", idle: "", working: "Working…", thinking: "Thinking longer…",
  done: "Done!", error: "Something went wrong", sleeping: "zzz",
};

let state: MascotState = initialState(Date.now());
let serverError = "";
function render() {
  label.textContent = serverError || LABELS[state.mood];
}

function apply(input: MascotEvent | { tick: number }) {
  state = reduce(state, input);
  render();
}

listen<MascotEvent>("mascot-event", (e) => apply(e.payload));
listen<string>("mascot-server-error", (e) => { serverError = e.payload; render(); });
invoke<string | null>("bridge_error").then((m) => { if (m) { serverError = m; render(); } });
setInterval(() => apply({ tick: Date.now() }), 250);
render();
