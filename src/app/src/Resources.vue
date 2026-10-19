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
            <h3 class="text-accent font-semibold">PPM Resources Info</h3>
            <div class="flex gap-1 ml-auto">
                <WSState class="ml-auto" :status="updates" />
                <RouterLink title="service list" :to="{ name: 'list' }"
                    ><Button class="border-none!"
                        ><ChevronLeft class="text-accent" /></Button
                ></RouterLink>
            </div>
        </template>
        <div class="w-full px-3 table table-auto text-sm">
            <div class="table-header-group">
                <div class="table-row">
                    <div class="table-cell">Id</div>
                    <div class="hidden sm:table-cell">Name</div>
                    <div class="table-cell">CPU</div>
                    <div class="table-cell">CPU time</div>
                    <div class="table-cell">I/O read</div>
                    <div class="table-cell">I/O write</div>
                    <div class="table-cell">Mem RSS</div>
                    <div class="table-cell">Mem VSZ</div>
                    <div class="hidden sm:table-cell">Uptime</div>
                </div>
            </div>
            <div
                :class="{ skel: !servicesStats }"
                class="table-row-group skel-anim"
            >
                <div
                    v-if="!isNil(daemonStats)"
                    key="daemon-stats"
                    class="table-row skel-anim fx-appear text-center hover:bg-foreground/5"
                >
                    <div class="table-cell"></div>
                    <div class="hidden sm:table-cell font-mono text-muted-foreground">
                        &lt;PPM daemon&gt;
                    </div>

                    <div class="table-cell">
                        {{ daemonStats?.cpu_usage?.toFixed(2) }}%
                    </div>
                    <div class="table-cell">
                        {{ formatDuration(daemonStats?.cpu_time) ?? "?" }}
                    </div>
                    <div class="table-cell">
                        <div>{{ formatBytes(daemonStats?.io_read) }}/sec</div>
                        <div>{{ formatBytes(daemonStats?.total_io_read) }}</div>
                    </div>
                    <div class="table-cell">
                        <div>{{ formatBytes(daemonStats?.io_write) }}/sec</div>
                        <div>
                            {{ formatBytes(daemonStats?.total_io_write) }}
                        </div>
                    </div>
                    <div class="table-cell">
                        {{ formatBytes(daemonStats?.mem_rss) }}
                    </div>
                    <div class="table-cell">
                        {{ formatBytes(daemonStats?.mem_vsz) }}
                    </div>
                    <div class="hidden sm:table-cell">
                        {{ formatDuration(daemonStats?.uptime) }}
                    </div>
                </div>
                <div
                    v-for="(info, id) in servicesStats || {
                        0: null,
                        1: null,
                        2: null,
                    }"
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
                >
                    <template v-if="servicesStats && isNil(info?.uptime)">
                        <div class="table-cell font-mono text-muted-foreground/60">
                            {{ id ?? "" }}
                        </div>
                        <div class="hidden sm:table-cell font-mono text-muted-foreground/60">
                            {{ serviceNameMap?.[id] ?? "" }}
                        </div>
                        <div class="table-cell"></div>
                        <div class="table-cell"></div>
                        <div class="table-cell"></div>
                        <div class="table-cell"></div>
                        <div class="table-cell"></div>
                        <div class="table-cell"></div>
                        <div class="hidden sm:table-cell"></div>
                    </template>
                    <template v-else>
                        <div class="table-cell font-mono">{{ id ?? "" }}</div>
                        <div class="hidden sm:table-cell font-mono">
                            {{ serviceNameMap?.[id] ?? "" }}
                        </div>
                        <div class="table-cell">
                            {{ info?.cpu_usage?.toFixed(2) }}%
                        </div>
                        <div class="table-cell">
                            {{ formatDuration(info?.cpu_time) ?? "?" }}
                        </div>
                        <div class="table-cell">
                            <div>{{ formatBytes(info?.io_read) }}/sec</div>
                            <div>{{ formatBytes(info?.total_io_read) }}</div>
                        </div>
                        <div class="table-cell">
                            <div>{{ formatBytes(info?.io_write) }}/sec</div>
                            <div>{{ formatBytes(info?.total_io_write) }}</div>
                        </div>
                        <div class="table-cell">
                            {{ formatBytes(info?.mem_rss) }}
                        </div>
                        <div class="table-cell">
                            {{ formatBytes(info?.mem_vsz) }}
                        </div>
                        <div class="hidden sm:table-cell">
                            {{ formatDuration(info?.uptime) }}
                        </div>
                    </template>
                </div>
            </div>
        </div>
    </Card>
</template>
