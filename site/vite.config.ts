// Builds the project website (GitHub Pages) from site/, reusing the app's mascot renderer.
import { defineConfig } from "vite";

export default defineConfig({
  root: "site",
  base: "./",
  build: { outDir: "../dist-site", emptyOutDir: true },
});
