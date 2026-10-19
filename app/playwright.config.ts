import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./test/e2e",
  fullyParallel: false,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:1421",
    channel: "msedge",
    headless: true,
    viewport: { width: 1440, height: 1000 },
    launchOptions: {args:["--force-high-performance-gpu","--use-fake-device-for-media-stream","--use-fake-ui-for-media-stream"]},
    screenshot: "only-on-failure",
  },
  webServer: {
    command:
      "npx vite --mode testbrowser --host 127.0.0.1 --port 1421 --strictPort",
    url: "http://127.0.0.1:1421",
    reuseExistingServer: !process.env.CI,
  },
  timeout: 30000,
});
