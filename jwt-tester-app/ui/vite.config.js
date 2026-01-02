import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  base: "/",
  build: {
    outDir: "dist",
    assetsDir: "assets",
    sourcemap: false,
  },
  test: {
    environment: "jsdom",
    setupFiles: "./src/test/setup.js",
    css: true,
    globals: true,
  },
  server: {
    proxy: {
      "/api": {
        target: process.env.JWT_TESTER_API_URL || "http://127.0.0.1:3000",
        changeOrigin: true,
      },
    },
  },
});
