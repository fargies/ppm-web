// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { makeRouter } from "./routes";

createApp(App).use(makeRouter()).mount("#app");
