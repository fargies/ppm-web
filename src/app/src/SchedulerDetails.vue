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
        <table aria-label="scheduler info" class="w-full px-3 table table-auto text-sm">
            <thead class="table-header-group">
                <tr>
                    <th class="table-cell">Id</th>
                    <th class="table-cell">Name</th>
                    <th class="table-cell">Event</th>
                    <th class="table-cell">Scheduled Time</th>
                </tr>
            </thead>
            <tbody
                :class="{ skel: !schedulerInfo }"
                class="table-row-group skel-anim"
            >
                <tr
                    v-for="(info, id) in schedulerInfo || {
                        0: null,
                        1: null,
                        2: null,
                    }"
                    role="link"
                    tabindex="0"
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
                    @keydown.enter="(e) => e.target.click()"
                >
                    <td aria-label="Id" class="table-cell font-mono">{{ info?.id ?? "" }}</td>
                    <td aria-label="Name" class="table-cell font-mono">
                        {{ serviceNameMap?.[info?.id] ?? "" }}
                    </td>
                    <td aria-label="Event" class="table-cell">
                        {{ info?.event ?? "?" }}
                    </td>
                    <td aria-label="Scheduled Time" class="table-cell">
                        {{ formatDate(info?.instant) ?? "" }}
                    </td>
                </tr>
            </tbody>
        </table>
    </Card>
</template>
