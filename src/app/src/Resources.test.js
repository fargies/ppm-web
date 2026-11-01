// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { expect, it, describe, vi } from "vitest";
import { render } from "vitest-browser-vue";
import App from "../src/App.vue";
import { router } from "../tests/setup";

describe("Resources", () => {
  async function prepareTest() {
    const screen = await render(App, {
      global: { plugins: [router] },
    });

    await screen.getByRole("button", { name: "resources" }).click();
    return screen;
  }

  it("shows service resources", async () => {
    const screen = await prepareTest();

    await expect
      .element(
        screen.getByLabelText("Mem RSS").getByText(/[0-9].*B$/).first(),
      )
      .toBeInTheDocument();
  });
});
