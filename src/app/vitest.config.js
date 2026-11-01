import { defineConfig } from "vitest/config";
import { playwright } from "@vitest/browser-playwright";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import commands from "./vitest.commands.js";

if (!process.env.PPM_LISTEN) {
  process.env.PPM_LISTEN = "127.0.0.1:5007";
}
if (!process.env.PPMWEB_LISTEN) {
  process.env.PPMWEB_LISTEN = "127.0.0.1:3007";
}

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  define: { __API_URL__: `"ws://${process.env.PPMWEB_LISTEN}/ws"` },
  test: {
    coverage: {
      provider: 'v8'
    },
    maxWorkers: 1,
    slowTestThreshold: 5000,
    browser: {
      commands: {
        ...commands,
      },
      enabled: true,
      provider: playwright(),
      instances: [{ browser: "chromium" }],

    },
  },
});
