<script setup>
import { ref } from "vue";
import { useInterval } from "@vueuse/core";
import { ChevronLeft } from "@lucide/vue";
import { Card, WSState, Button } from "./components";
import { isNil, formatDate } from "./utils";
import { client } from "./Client";
import { serviceNameMap } from "./store";
import { DateTime } from "luxon";

const updates = ref(0);
const schedulerInfo = ref(null);

client.onMessage = (data) => {
    if (data?.id === "scheduler") {
        updates.value += 1;
        let update_list = false;
        for (const [id, info] of Object.entries(data.result)) {
            if (info?.instant) {
                info.instant = DateTime.fromISO(info.instant);
            }
            if (schedulerInfo.value === null) {
                schedulerInfo.value = {};
            }
            if (
                !isNil(info?.id) &&
                !serviceNameMap.value?.hasOwnProperty(info.id)
            ) {
                update_list = true;
            }
            schedulerInfo.value[id] = info;
        }
        if (update_list) {
            client.request_list();
        }
    }
};

client.send({ id: "scheduler", method: "show-scheduler" });
useInterval(5000, {
    controls: true,
    callback: () => client.send({ id: "scheduler", method: "show-scheduler" }),
});
</script>
<template>
    <Card class="col-span-12">
        <template #header
            ><h3>PPM Scheduler Info</h3>
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
        <div class="w-full px-3 table table-auto text-sm">
            <div class="table-header-group">
                <div class="table-row">
                    <div class="table-cell">Id</div>
                    <div class="table-cell">Name</div>
                    <div class="table-cell">Event</div>
                    <div class="table-cell">Scheduled Time</div>
                </div>
            </div>
            <div
                :class="{ skel: !schedulerInfo }"
                class="table-row-group skel-anim"
            >
                <div
                    v-for="(info, id) in schedulerInfo || {
                        0: null,
                        1: null,
                        2: null,
                    }"
                    :key="id"
                    :class="{
                        'fx-appear': schedulerInfo,
                        'cursor-pointer': !isNil(info?.id),
                    }"
                    class="table-row text-center hover:bg-foreground/5"
                    @click="
                        if (!isNil(info?.id)) {
                            $router.push({
                                name: 'service',
                                params: { id: info?.id },
                            });
                        }
                    "
                >
                    <div class="table-cell font-mono">{{ info?.id ?? "" }}</div>
                    <div class="table-cell font-mono">
                        {{ serviceNameMap?.[info?.id] ?? "" }}
                    </div>
                    <div class="table-cell">
                        {{ info?.event ?? "?" }}
                    </div>
                    <div class="table-cell">
                        {{ formatDate(info?.instant) ?? "" }}
                    </div>
                </div>
            </div>
        </div>
    </Card>
</template>
