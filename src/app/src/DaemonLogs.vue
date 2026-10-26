<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { ref } from "vue";
import { ChevronLeft, ToggleRight, ToggleLeft, Pause, Play, Infinity } from "@lucide/vue";
import { Card, WSState, Button } from "./components";

import ServiceLog from "./components/ServiceLog.vue";

const updates = ref(0);
const paused = ref(false);
const preserve_lines = ref(false);
const max_lines = ref(20);
</script>
<template>
    <Card class="col-span-12">
        <template #header>
            <h3>PPM Daemon Logs</h3>
            <div class="flex gap-1 ml-auto align-middle items-center">
                <WSState class="ml-auto" :status="updates" />
                <Button
                    @click="paused = !paused"
                    title="pause log stream"
                    :class="{ active: paused }"
                    ><template v-if="paused"> <Play /> Continue </template>
                    <template v-else><Pause />Pause</template>
                </Button>
                <div title="log lines to keep" class="button-block">
                    <Button :active="max_lines === 20" @click="max_lines = 20">20</Button>
                    <Button :active="max_lines === 80" @click="max_lines = 80">80</Button>
                    <Button :active="max_lines === null" aria-label="infinite" @click="max_lines = null"><Infinity/></Button>
                </div>
                <RouterLink :to="{ name: 'list' }" custom v-slot="{ navigate }"
                    ><Button
                        @click="navigate"
                        title="service list"
                        class="border-none! h-fill!"
                        ><ChevronLeft class="text-accent" /></Button
                ></RouterLink>
            </div>
        </template>
        <div class="w-full">
            <ServiceLog
                id="ppm-daemon"
                :max_lines="max_lines"
                :lines="20"
                :follow="true"
                :paused="paused"
                @update="updates += 1"
            />
        </div>
    </Card>
</template>
