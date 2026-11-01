// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { expect, it, describe, vi } from "vitest";
import { render } from "vitest-browser-vue";
import App from "../src/App.vue";
import { router } from "../tests/setup";

describe("DaemonLogs", () => {
  async function prepareTest() {
    const screen = await render(App, {
      global: { plugins: [router] },
    });

    await screen.getByRole("button", { name: "server logs" }).click();
    return screen;
  }

  it("shows daemon logs", async () => {
    const screen = await prepareTest();

    await expect
      .element(screen.getByLabelText("log line").first())
      .toBeInTheDocument();
  });

  it("limit log pages", async () => {
    const screen = await prepareTest();

    let loglines = screen.getByLabelText("log line");
    await expect.element(loglines.first()).toBeInTheDocument();

    await screen.getByRole("button", { name: "20" }).click();
    await vi.waitFor(() => expect(loglines.elements()).toHaveLength(20), {
      timeout: 5000,
    });

    await screen.getByRole("button", { name: "80" }).click();
    await vi.waitFor(() => expect(loglines.elements()).toHaveLength(80), {
      timeout: 5000,
    });

    await screen.getByRole("button", { name: "20" }).click();
    await vi.waitFor(() => expect(loglines.elements()).toHaveLength(20), {
      timeout: 5000,
    });
  });
});
