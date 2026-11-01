// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { expect, it, describe } from "vitest";
import { render } from "vitest-browser-vue";
import App from "../src/App.vue";
import { client } from "../src/Client";
import { router } from "../tests/setup";

describe("ServiceDetails", () => {
  async function prepareTest(id) {
    const screen = await render(App, {
      global: { plugins: [router] },
    });

    let elt = screen.getByRole("link").getByLabelText("Id").getByText(id);
    await expect.element(elt).toBeInTheDocument();
    await elt.click();
    return screen;
  }

  it("shows service details", async () => {
    const screen = await prepareTest("1");

    await expect.element(screen.getByLabelText("Controls")).toBeInTheDocument();
    await expect
      .element(screen.getByLabelText("Name").getByText("periodic_write"))
      .toBeInTheDocument();
  });

  it("can start/stop service", async () => {
    const screen = await prepareTest("1");

    await expect.element(screen.getByLabelText("Controls")).toBeInTheDocument();
    await expect
      .element(screen.getByLabelText("Name").getByText("periodic_write"))
      .toBeInTheDocument();

    await screen.getByRole("button", { name: "Run" }).click();
    await expect
      .element(screen.getByLabelText("Status").getByText("Running"))
      .toBeInTheDocument();

    await screen.getByRole("button", { name: "Stop" }).click();
    client.request_info(null);
    await expect
      .element(screen.getByLabelText("Status").getByText("Finished"))
      .toBeInTheDocument();

    await screen.getByRole("button", { name: "Run" }).click();
    client.request_info(null);
    await expect
      .element(screen.getByLabelText("Status").getByText("Running"))
      .toBeInTheDocument();
  });

  it("can navigate services", async () => {
    const screen = await prepareTest("1");

    let id = screen
      .getByLabelText("Id")
      .getByText(/^[0-9]+$/)
      .element().textContent;
    await screen.getByRole("button", { name: "next service" }).click();
    let new_id = screen
      .getByLabelText("Id")
      .getByText(/^[0-9]+$/)
      .element().textContent;
    expect(id).not.toEqual(new_id);

    await screen.getByRole("button", { name: "previous service" }).click();
    new_id = screen
      .getByLabelText("Id")
      .getByText(/^[0-9]+$/)
      .element().textContent;
    expect(id).toEqual(new_id);
  });

  it("can show logs", async () => {
    const screen = await prepareTest("1");

    /* open logs */
    let logs = screen.getByLabelText("Logs");
    await expect.element(logs).toBeInTheDocument();
    if (!logs.element().open) {
      await logs.click();
      await expect.element(logs).toHaveAttribute("open");
    }

    await expect
      .element(logs.getByText(/records in/).first())
      .toBeInTheDocument();
  });

  it("can show resources", async () => {
    const screen = await prepareTest("1");

    /* open logs */
    let elt = screen.getByLabelText("Resources");
    await expect.element(elt).toBeInTheDocument();
    if (!elt.element().open) {
      await elt.click();
      await expect.element(elt).toHaveAttribute("open");
    }

    await expect
      .element(
        elt.getByRole("status", { name: "Mem RSS" }).getByText(/[0-9].*B$/),
      )
      .toBeInTheDocument();
  });
});
