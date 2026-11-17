// Overlay entry point. Task 1 renders a placeholder; later tasks swap in Ember and the mood reducer.
const stage = document.getElementById("stage")!;

const style = document.createElement("style");
style.textContent = `
  #stage { position: fixed; right: 12px; bottom: 12px; }
  .placeholder { font-size: 128px; line-height: 1; animation: bob 1.6s ease-in-out infinite; }
  @keyframes bob { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-6px); } }
  @media (prefers-reduced-motion: reduce) { .placeholder { animation: none; } }
`;
document.head.appendChild(style);

const mascot = document.createElement("div");
mascot.className = "placeholder";
mascot.textContent = "🔥";
stage.appendChild(mascot);
