<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import logoUrl from "../../../deps/ppm/data/ppm.svg";
import {
    BadgeAlert,
    BadgeCheck,
    GlobeCheck,
    GlobeX,
    Link,
    Moon,
    Sun,
    Unlink,
} from "@lucide/vue";
import { theme, toggleTheme } from "./store";

import { Button, ErrorsList } from "./components";

import { client } from "./Client";
import { computed } from "vue";

const connected = computed(() => client?.status?.value === "OPEN");
</script>
<template>
    <nav
        class="sticky z-100 flex w-full justify-between mb-1 px-1 shadow-sm gap-2 top-0 left-0 items-center border-b border-border bg-background"
    >
        <RouterLink class="flex gap-1 items-center" :to="{ name: 'list' }">
            <img id="logo" class="w-12 h-12" :src="logoUrl" />
            <h1 class="text-accent text-2xl font-bold">PPM</h1>
        </RouterLink>
        <div class="flex gap-2 align-middle">
            <span
                :title="`connection status: ${client?.status?.value}`"
                :class="{ 'fx-blink': !connected }"
                aria-label="connection status"
            >
                <GlobeCheck
                    v-if="connected"
                    class="fx-appear text-success/40"
                />
                <GlobeX v-else class="fx-blink text-failure/60" />
            </span>
            <Button title="theme switch" @click="toggleTheme"
                ><Moon v-if="theme === 'dark'" /><Sun v-else
            /></Button>
        </div>
    </nav>
    <div class="grid grid-cols-12 gap-1 col-span-12 p-1">
        <RouterView />
        <ErrorsList />
    </div>
</template>
