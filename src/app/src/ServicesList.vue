<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { ref } from "vue";
import { useInterval } from "@vueuse/core";
import WSState from "./components/WSState.vue";
import Card from "./components/Card.vue";
import Badge from "./components/Badge.vue";
import {
    Activity,
    CheckSquare,
    Logs,
    RotateCcw,
    Skull,
    SquareX,
    Timer,
} from "@lucide/vue";
import { servicesInfo, serviceNameMap, now } from "./store";
import { client } from "./Client";
import { formatDate, upTime as utilsUpTime, mapStatusBadge } from "./utils";
import { Button } from "./components";

const updates = ref(0);
const uptime_width = ref(0);

client.onMessage = (_data) => {
    updates.value += 1;
};

client.request_clock_check();
client.request_info(null);
useInterval(5000, {
    callback: () => client.request_info(null),
});

function upTime(start_time, end_time) {
    let ret = utilsUpTime(start_time, end_time);

    if (ret?.length >= uptime_width.value) {
        uptime_width.value = ret.length + 1;
    }
    return ret;
}
</script>
<template>
    <Card class="col-span-12">
        <template #header>
            <h3>PPM Service Info</h3>
            <WSState class="ml-auto" :status="updates" />
        </template>
        <table role="table" class="w-full px-3 table table-auto text-sm">
            <thead class="table-header-group">
                <tr>
                    <th class="table-cell">Id</th>
                    <th class="sm:table-cell hidden">Name</th>
                    <th class="sm:table-cell hidden">Pid</th>
                    <th class="table-cell">Active</th>
                    <th class="table-cell">Status</th>
                    <th class="sm:table-cell hidden">Start time</th>
                    <th class="table-cell">Uptime</th>
                    <th
                        aria-label="Restarts"
                        title="Restarts"
                        class="table-cell"
                    >
                        <RotateCcw aria-label="Restarts" />
                    </th>
                    <th aria-label="Crashes" title="Crashes" class="table-cell">
                        <Skull arial-label="Crashes" />
                    </th>
                </tr>
            </thead>
            <tbody
                :class="{ skel: !servicesInfo }"
                class="table-row-group skel-anim"
            >
                <tr
                    v-for="(info, id) in servicesInfo || {
                        0: null,
                        1: null,
                        2: null,
                    }"
                    :role="info === null ? 'placeholder' : 'link'"
                    tabindex="0"
                    :key="id"
                    :class="{ 'fx-appear': servicesInfo }"
                    class="table-row text-center hover:bg-foreground/5 cursor-pointer"
                    @click="
                        if (info !== null) {
                            $router.push({
                                name: 'service',
                                params: { id },
                            });
                        }
                    "
                    @keydown.enter="(e) => e.target.click()"
                >
                    <td aria-label="Id" class="table-cell font-mono">
                        {{ id }}
                    </td>
                    <td aria-label="Name" class="sm:table-cell hidden font-mono">
                        {{ serviceNameMap?.[id] ?? "?" }}
                    </td>
                    <td aria-label="Pid" class="hidden sm:table-cell font-mono">
                        {{ info?.pid }}
                    </td>
                    <td
                        aria-label="Active"
                        :aria-valuenow="info?.active"
                        class="table-cell"
                    >
                        <CheckSquare class="text-success" v-if="info?.active" />
                        <SquareX v-else class="text-failure" />
                    </td>
                    <td
                        aria-label="Status"
                        :aria-valuenow="info?.status"
                        class="table-cell"
                    >
                        <Badge :class="mapStatusBadge(info?.status)">{{
                            info?.status
                        }}</Badge>
                    </td>
                    <td aria-label="Start time" class="sm:table-cell hidden">
                        {{ formatDate(info?.start_time) ?? "?" }}
                    </td>
                    <td
                        class="table-cell"
                        aria-label="Uptime"
                        :style="`min-width: ${uptime_width}ch`"
                    >
                        {{
                            upTime(info?.start_time, info?.end_time ?? now) ??
                            "?"
                        }}
                    </td>
                    <td aria-label="Restarts" class="table-cell">
                        {{ info?.restarts }}
                    </td>
                    <td aria-label="Crashes" class="table-cell">
                        {{ info?.crashed }}
                    </td>
                </tr>
            </tbody>
        </table>
        <div class="fx-appear flex p-1 gap-1 justify-end">
            <RouterLink
                :to="{ name: 'daemonlogs' }"
                custom
                v-slot="{ navigate }"
            >
                <Button
                    @click="navigate"
                    class="text-xs"
                    title="server logs"
                    label="server logs"
                    ><Logs />Server Logs
                </Button>
            </RouterLink>
            <RouterLink
                :to="{ name: 'scheduler' }"
                custom
                v-slot="{ navigate }"
            >
                <Button
                    @click="navigate"
                    class="text-xs"
                    title="scheduler information"
                    label="scheduler"
                    ><Timer />Scheduler
                </Button>
            </RouterLink>
            <RouterLink
                :to="{ name: 'resources' }"
                custom
                v-slot="{ navigate }"
            >
                <Button
                    @click="navigate"
                    class="text-xs"
                    title="resources statistics"
                    label="resources"
                    ><Activity />Resources
                </Button>
            </RouterLink>
        </div>
    </Card>
</template>
