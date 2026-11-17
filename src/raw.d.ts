// Vite's `?raw` imports give a file's text (used by tests that check shipped scripts).
declare module "*?raw" {
  const text: string;
  export default text;
}
