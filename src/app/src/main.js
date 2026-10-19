// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import "./style.css";
import App from "./App.vue";

import ServicesList from "./ServicesList.vue";
import ServiceDetails from "./ServiceDetails.vue";
import SchedulerDetails from "./SchedulerDetails.vue";
import Resources from "./Resources.vue";

const routes = [
  { name: "list", path: "/", component: ServicesList },
  {
    name: "service",
    path: "/service/:id",
    component: ServiceDetails,
    props: true,
  },
  { name: "scheduler", path: "/scheduler", component: SchedulerDetails },
  { name: "resources", path: "/resources", component: Resources },
];

createApp(App)
  .use(createRouter({ history: createWebHistory(), routes }))
  .mount("#app");
