// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { expect, it, describe, vi } from "vitest";
import { render } from "vitest-browser-vue";
import App from "../src/App.vue";
import { router } from "../tests/setup";

describe("Scheduler", () => {
  async function prepareTest() {
    const screen = await render(App, {
      global: { plugins: [router] },
    });

    await screen.getByRole("button", { name: "scheduler" }).click();
    return screen;
  }

  it("shows scheduler info", async () => {
    const screen = await prepareTest();

    await expect
      .element(screen.getByLabelText("Event").getByText("Sysinfo"))
      .toBeInTheDocument();
    await expect
      .element(screen.getByLabelText("Event").getByText("ClockCheck"))
      .toBeInTheDocument();
  });
});
