import { defineConfig } from "vite";
export default defineConfig({
  base: "/openworks/",
  define: { global: "globalThis" },
  build: { target: "es2022", chunkSizeWarningLimit: 3000 },
});
