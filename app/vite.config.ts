import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  optimizeDeps: {
    include: [
      "ajv/dist/runtime/ucs2length.js",
      "three/addons/loaders/GLTFLoader.js",
      "three/addons/lights/RectAreaLightUniformsLib.js",
    ],
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./test/setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
  },
});
