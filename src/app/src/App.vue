<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import logoUrl from "../../../deps/ppm/data/ppm.svg";
import { Moon, Sun } from "@lucide/vue";
import { theme, toggleTheme } from "./store";

import { Button, ErrorsList } from "./components";
</script>
<template>
    <nav
        class="sticky z-100 flex w-full justify-between mb-1 px-1 shadow-sm gap-2 top-0 left-0 items-center border-b border-border bg-background"
    >
        <RouterLink class="flex gap-1 items-center" :to="{ name: 'list' }">
            <img id="logo" class="w-12 h-12" :src="logoUrl" />
            <h1 class="text-accent text-2xl font-bold">PPM</h1>
        </RouterLink>
        <Button title="theme switch" @click="toggleTheme"
            ><Moon v-if="theme === 'dark'" /><Sun v-else
        /></Button>
    </nav>
    <div class="grid grid-cols-12 gap-1 col-span-12 p-1">
        <RouterView />
        <ErrorsList />
    </div>
</template>
