// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { beforeAll, afterAll, expect, it, describe, afterEach } from "vitest";
import { render } from "vitest-browser-vue";
import App from "../src/App.vue";
import { client } from "../src/Client";

import { router } from "./setup";

describe("PPM navigation tests", () => {
  it("renders service list", async () => {
    const screen = await render(App, {
      global: { plugins: [router] },
    });

    await expect
      .element(
        screen
          .getByRole("link")
          .getByLabelText("Name")
          .filter({ hasNotText: "?" })
          .first(),
      )
      .toBeInTheDocument();
  });
});
