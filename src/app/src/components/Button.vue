<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

defineProps({
  label: { type: String, default: "" },
    active: { type: Boolean, default: false },
    skel: { type: Boolean, default: false },
});
</script>
<template>
    <button
        :label="label"
        :aria-label="label"
        class="skel-anim w-fit h-fill duration-300 shrink-0 transition-all"
        :class="{ skel, active }"
    >
        <slot />
    </button>
</template>
