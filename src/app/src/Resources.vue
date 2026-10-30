<script setup>
import { Card, WSState, Button } from "./components";
import { ref } from "vue";
import { ChevronLeft } from "@lucide/vue";
import { servicesStats, serviceNameMap } from "./store";
import { client } from "./Client";
import { useInterval } from "@vueuse/core";
import { isNil, formatDuration, formatBytes } from "./utils";

const updates = ref(0);
const daemonStats = ref(null);

client.request_stats();
client.send({ id: "daemon-stats", method: "daemon-stats" });
useInterval(5000, {
    controls: true,
    callback: () => {
        client.request_stats();
        client.send({ id: "daemon-stats", method: "daemon-stats" });
    },
});

client.onMessage = (data) => {
    if (data?.id === "stats") {
        updates.value += 1;
    } else if (data?.id === "daemon-stats") {
        daemonStats.value = client.prepare_stats(data.result);
    }
};
</script>
<style scoped>
.disabled {
    opacity: 0.5 !important;
}
</style>
<template>
    <Card class="col-span-12">
        <template #header>
            <h3>PPM Resources Info</h3>
            <div class="flex gap-1 ml-auto">
                <WSState class="ml-auto" :status="updates" />
                <RouterLink :to="{ name: 'list' }" custom v-slot="{ navigate }"
                    ><Button
                        @click="navigate"
                        title="service list"
                        class="border-none!"
                        ><ChevronLeft class="text-accent" /></Button
                ></RouterLink>
            </div>
        </template>
        <table role="table" class="w-full px-3 table table-auto text-sm">
            <thead class="table-header-group">
                <tr class="table-row">
                    <th class="table-cell">Id</th>
                    <th class="hidden sm:table-cell">Name</th>
                    <th class="table-cell">CPU</th>
                    <th class="table-cell">CPU time</th>
                    <th class="table-cell">I/O read</th>
                    <th class="table-cell">I/O write</th>
                    <th class="table-cell">Mem RSS</th>
                    <th class="table-cell">Mem VSZ</th>
                    <th class="hidden sm:table-cell">Uptime</th>
                </tr>
            </thead>
            <tbody
                :class="{ skel: !servicesStats }"
                class="table-row-group skel-anim"
            >
                <tr
                    v-if="!isNil(daemonStats)"
                    key="daemon-stats"
                    class="table-row skel-anim fx-appear text-center hover:bg-foreground/5"
                >
                    <td class="table-cell"></td>
                    <td
                        class="hidden sm:table-cell font-mono text-muted-foreground"
                    >
                        &lt;PPM daemon&gt;
                    </td>

                    <td class="table-cell">
                        {{ daemonStats?.cpu_usage?.toFixed(2) }}%
                    </td>
                    <td class="table-cell">
                        {{ formatDuration(daemonStats?.cpu_time) ?? "?" }}
                    </td>
                    <td class="table-cell">
                        <div>{{ formatBytes(daemonStats?.io_read) }}/sec</div>
                        <div>{{ formatBytes(daemonStats?.total_io_read) }}</div>
                    </td>
                    <td class="table-cell">
                        <div>{{ formatBytes(daemonStats?.io_write) }}/sec</div>
                        <div>
                            {{ formatBytes(daemonStats?.total_io_write) }}
                        </div>
                    </td>
                    <td class="table-cell">
                        {{ formatBytes(daemonStats?.mem_rss) }}
                    </td>
                    <td class="table-cell">
                        {{ formatBytes(daemonStats?.mem_vsz) }}
                    </td>
                    <td class="hidden sm:table-cell">
                        {{ formatDuration(daemonStats?.uptime) }}
                    </td>
                </tr>
                <tr
                    v-for="(info, id) in servicesStats || {
                        0: null,
                        1: null,
                        2: null,
                    }"
                    role="link"
                    tabindex="0"
                    :key="id"
                    :class="{
                        'fx-appear': servicesStats,
                        'cursor-pointer': !isNil(info),
                        // disabled: servicesStats && isNil(info?.uptime),
                    }"
                    class="table-row text-center hover:bg-foreground/5"
                    @click="
                        if (!isNil(info)) {
                            $router.push({
                                name: 'service',
                                params: { id },
                            });
                        }
                    "
                    @keydown.enter="(e) => e.target.click()"
                >
                    <template v-if="servicesStats && isNil(info?.uptime)">
                        <td
                            class="table-cell font-mono text-muted-foreground/60"
                        >
                            {{ id ?? "" }}
                        </td>
                        <td
                            class="hidden sm:table-cell font-mono text-muted-foreground/60"
                        >
                            {{ serviceNameMap?.[id] ?? "" }}
                        </td>
                        <td class="table-cell"></td>
                        <td class="table-cell"></td>
                        <td class="table-cell"></td>
                        <td class="table-cell"></td>
                        <td class="table-cell"></td>
                        <td class="table-cell"></td>
                        <td class="hidden sm:table-cell"></td>
                    </template>
                    <template v-else>
                        <td class="table-cell font-mono">{{ id ?? "" }}</td>
                        <td class="hidden sm:table-cell font-mono">
                            {{ serviceNameMap?.[id] ?? "" }}
                        </td>
                        <td class="table-cell">
                            {{ info?.cpu_usage?.toFixed(2) }}%
                        </td>
                        <td class="table-cell">
                            {{ formatDuration(info?.cpu_time) ?? "?" }}
                        </td>
                        <td class="table-cell">
                            <div>{{ formatBytes(info?.io_read) }}/sec</div>
                            <div>{{ formatBytes(info?.total_io_read) }}</div>
                        </td>
                        <td class="table-cell">
                            <div>{{ formatBytes(info?.io_write) }}/sec</div>
                            <div>{{ formatBytes(info?.total_io_write) }}</div>
                        </td>
                        <td class="table-cell">
                            {{ formatBytes(info?.mem_rss) }}
                        </td>
                        <td class="table-cell">
                            {{ formatBytes(info?.mem_vsz) }}
                        </td>
                        <td class="hidden sm:table-cell">
                            {{ formatDuration(info?.uptime) }}
                        </td>
                    </template>
                </tr>
            </tbody>
        </table>
    </Card>
</template>
