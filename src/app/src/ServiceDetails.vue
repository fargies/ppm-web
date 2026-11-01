<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { ref, computed, watch } from "vue";
import { useInterval } from "@vueuse/core";
import {
    RotateCcw,
    Square,
    Play,
    ChevronRight,
    ChevronLeft,
    ChevronUp,
    CheckSquare,
    SquareX,
    Infinity,
    Pause,
} from "@lucide/vue";
import { client } from "./Client";
import {
    servicesInfo,
    serviceNameMap,
    now,
    servicesStats,
    save,
    restore,
} from "./store";
import {
    Card,
    WSState,
    ServiceInfoPart,
    Badge,
    Button,
    Collapsible,
} from "./components";
import {
    mapStatusBadge,
    upTime,
    formatDuration,
    formatBytes,
    isNil,
} from "./utils";
import ServiceLog from "./components/ServiceLog.vue";

let { id } = defineProps({ id: String });

const info_updates = ref(0);
const stats_updates = ref(0);
const log_updates = ref(0);
const paused = ref(false);
const max_lines = ref(restore("service_log_max_lines", 20));

const info = computed(() => servicesInfo.value?.[id]);
const stats = computed(() => servicesStats.value?.[id]);
const nextService = computed(() =>
    Object.keys(serviceNameMap.value).find((i) => i > id),
);
const prevService = computed(() =>
    Object.keys(serviceNameMap.value).findLast((i) => i < id),
);

watch(max_lines, (max_lines) => save("service_log_max_lines", max_lines));

client.onMessage = (data) => {
    if (data?.id === "info") {
        info_updates.value += 1;
    } else if (data?.id === "stats") {
        stats_updates.value += 1;
    }
};
client.request_info(id);
const interval = useInterval(5000, {
    callback: () => {
        client.request_info(id);
        client.request_stats(id);
    },
});
</script>

<template>
    <Card class="col-span-12">
        <template #header>
            <h3
                title="service name"
                :class="{ skel: !serviceNameMap?.[id] }"
                class="skel-anim text-2xl"
            >
                {{ serviceNameMap?.[id] ?? "placeholder" }}
            </h3>
            <div class="flex gap-1 ml-auto">
                <RouterLink
                    :to="{ params: { id: prevService } }"
                    custom
                    v-slot="{ navigate }"
                    ><Button
                        @click="navigate"
                        title="previous service"
                        :disabled="isNil(prevService)"
                        class="border-none!"
                        ><ChevronLeft class="text-accent" /></Button
                ></RouterLink>
                <RouterLink :to="{ name: 'list' }" custom v-slot="{ navigate }"
                    ><Button
                        @click="navigate"
                        title="service list"
                        class="border-none!"
                        ><ChevronUp class="text-accent" /></Button
                ></RouterLink>
                <RouterLink
                    :to="{ params: { id: nextService } }"
                    custom
                    v-slot="{ navigate }"
                    ><Button
                        @click="navigate"
                        title="next service"
                        :disabled="isNil(nextService)"
                        class="border-none!"
                        ><ChevronRight class="text-accent" /></Button
                ></RouterLink>
            </div>
        </template>
        <Collapsible aria-label="Controls" :open="true">
            <template #summary
                ><span title="service controls">Controls</span></template
            >
            <div class="flex gap-1 justify-center md:justify-start md:ps-2">
                <Button
                    :skel="!info"
                    :disabled="!info?.active"
                    title="stop and deactivate the service"
                    label="Stop"
                    @click="
                        client.send({
                            id: 'stop',
                            method: 'stop',
                            params: { service: id },
                        })
                    "
                >
                    <Square />Stop
                </Button>
                <Button
                    :skel="!info"
                    label="Run"
                    title="(re)start and activate the service"
                    @click="
                        client.send({
                            id: 'restart',
                            method: 'restart',
                            params: { service: id },
                        })
                    "
                >
                    <template v-if="info?.pid"><RotateCcw /> Restart</template>
                    <template v-else> <Play /> Start</template>
                </Button>
            </div>
        </Collapsible>
        <Collapsible aria-label="Status" :open="true">
            <template #summary
                ><div
                    title="service live status"
                    class="grow flex justify-between"
                >
                    Status <WSState :status="info_updates" /></div
            ></template>
            <div
                class="grid grid-cols-1 sm:grid-cols-3 lg:grid-cols-6 gap-1 p-1"
            >
                <ServiceInfoPart :skel="!info" title="Id">{{
                    id
                }}</ServiceInfoPart>
                <ServiceInfoPart
                    :skel="!info"
                    class="sm:col-span-2 lg:col-span-5"
                    title="Name"
                    >{{ serviceNameMap?.[id] || "?" }}</ServiceInfoPart
                >
                <ServiceInfoPart :skel="!info" title="Status"
                    ><Badge :class="mapStatusBadge(info?.status)">{{
                        info?.status ?? "?"
                    }}</Badge></ServiceInfoPart
                >
                <ServiceInfoPart :skel="!info" title="Active">
                    <CheckSquare class="text-success" v-if="info?.active" />
                    <SquareX v-else class="text-failure" />
                </ServiceInfoPart>
                <ServiceInfoPart :skel="!info" title="Pid">{{
                    info?.pid ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!info" title="Uptime">{{
                    upTime(info?.start_time, info?.end_time ?? now) ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!info" title="Restarts">{{
                    info?.restarts ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!info" title="Crashes">{{
                    info?.crashed ?? "?"
                }}</ServiceInfoPart>
            </div>
        </Collapsible>
        <Collapsible aria-label="Resources" v-show="info?.pid">
            <template #summary
                ><div
                    title="service resource usage statistics"
                    class="grow flex justify-between"
                >
                    Resources<WSState :status="stats_updates" /></div
            ></template>
            <div
                class="grid grid-cols-1 sm:grid-cols-3 lg:grid-cols-6 gap-1 p-1"
            >
                <ServiceInfoPart :skel="!stats" title="CPU"
                    >{{ stats?.cpu_usage?.toFixed(2) }}%</ServiceInfoPart
                >
                <ServiceInfoPart :skel="!stats" title="Cpu Time">{{
                    formatDuration(stats?.cpu_time) ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="I/O read">
                    {{ formatBytes(stats?.io_read) ?? "?" }}/sec
                </ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="I/O write"
                    >{{ formatBytes(stats?.io_write) ?? "?" }}/sec
                </ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="I/O read">{{
                    formatBytes(stats?.total_io_read) ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="I/O write">{{
                    formatBytes(stats?.total_io_write) ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="Mem RSS">{{
                    formatBytes(stats?.mem_rss) ?? "?"
                }}</ServiceInfoPart>
                <ServiceInfoPart :skel="!stats" title="Mem VSZ">{{
                    formatBytes(stats?.mem_vsz) ?? "?"
                }}</ServiceInfoPart>
            </div>
        </Collapsible>
        <Collapsible aria-label="Logs">
            <template #summary="{ open }">
                <div title="service logs" class="grow flex justify-between">
                    Logs
                    <div class="flex gap-1 ml-auto align-middle items-center">
                        <WSState :status="log_updates" />
                        <Button
                            v-show="open"
                            @click.stop="paused = !paused"
                            title="pause log stream"
                            class="fx-appear"
                            :class="{ active: paused }"
                            ><template v-if="paused">
                                <Play /> Continue
                            </template>
                            <template v-else><Pause />Pause</template>
                        </Button>
                        <div
                            v-show="open"
                            title="log lines to keep"
                            class="fx-appear button-block"
                        >
                            <Button
                                :active="max_lines === 20"
                                @click.stop="max_lines = 20"
                                >20</Button
                            >
                            <Button
                                :active="max_lines === 80"
                                @click.stop="max_lines = 80"
                                >80</Button
                            >
                            <Button
                                :active="max_lines === null"
                                aria-label="infinite"
                                @click.stop="max_lines = null"
                                ><Infinity
                            /></Button>
                        </div>
                    </div>
                </div>
            </template>
            <template #default="{ open }">
                <ServiceLog
                    v-if="open"
                    :id="id"
                    :max_lines="max_lines"
                    :lines="max_lines"
                    :paused="paused"
                    :follow="true"
                    @update="log_updates += 1"
                />
            </template>
        </Collapsible>
    </Card>
</template>
