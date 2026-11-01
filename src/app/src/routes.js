// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import ServicesList from "./ServicesList.vue";
import ServiceDetails from "./ServiceDetails.vue";
import SchedulerDetails from "./SchedulerDetails.vue";
import Resources from "./Resources.vue";
import DaemonLogs from "./DaemonLogs.vue";
import { createRouter, createWebHistory } from "vue-router";

export const routes = [
  { name: "list", path: "/", component: ServicesList },
  {
    name: "service",
    path: "/service/:id",
    component: ServiceDetails,
    props: true,
  },
  { name: "scheduler", path: "/scheduler", component: SchedulerDetails },
  { name: "resources", path: "/resources", component: Resources },
  { name: "daemonlogs", path: "/daemonlogs", component: DaemonLogs },
];

export function makeRouter() {
  return createRouter({
    history: createWebHistory(),
    routes,
  });
}
