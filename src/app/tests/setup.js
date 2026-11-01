// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { beforeAll, afterAll, afterEach } from "vitest";
import "../src/style.css";
import { commands } from "vitest/browser";
import { makeRouter } from "../src/routes";

beforeAll(async () => {
  await commands.startServer();
});

afterEach(() => router.push("/"));

afterAll(async () => {
  await commands.stopServer();
});

export const router = makeRouter();
