<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>
import { parseAnsiSequences } from "ansi-sequence-parser";
import { ref, watch } from "vue";
import { palette } from "./LogLinePalette";

const { line } = defineProps({ line: { type: String, required: true } });
const parsed = ref(null);

/**
 * @typedef {import("ansi-sequence-parser").ParseToken} ParseToken
 * @typedef {{ value: String, foreground?: String | null, background?: String | null, class?: any }} LogLineItem
 */

const date_re = /^[0-9]{4}-[0-9]{2}-[0-9]{2}T/;
const ppm_log_items = {
    TRACE: { value: "TRACE", class: { "loglevel": true, "loglevel-trace": true } },
    " INFO": { value: " INFO", class: { "loglevel": true, "loglevel-info": true } },
    DEBUG: { value: "DEBUG", class: { "loglevel": true, "loglevel-debug": true } },
    " WARN": { value: " WARN", class: { "loglevel": true, "loglevel-warn": true } },
    ERROR: { value: "ERROR", class: { "loglevel": true, "loglevel-error": true } },
};

/**
 *
 * @param line {String}
 * @param out {LogLineItem[]}
 */
function parse_ppm_log(line, out) {
    let ret = line;
    if (date_re.test(ret)) {
        let date_end = ret.indexOf(" ") + 1;
        out.push({
            value: ret.slice(0, date_end),
            class: { "logdate": true },
        });
        ret = ret.slice(date_end);
        for (const [key, value] of Object.entries(ppm_log_items)) {
            console.log(key, ret.startsWith(key), ret);
            if (ret.startsWith(key)) {
                out.push(value);
                ret = ret.slice(key.length);
                break;
            }
        }
    }
    return ret;
}

watch(
    () => line,
    () => {
        let items = /** @type {LogLineItem[]} */ ([]);
        let log_line = parse_ppm_log(line, items);
        for (const token of parseAnsiSequences(log_line)) {
            items.push({
                value: token.value,
                foreground: token.foreground ? palette(token.foreground) : null,
                background: token.background ? palette(token.background) : null,
                class: {
                    "log-bold": token.decorations?.has("bold"),
                    "log-muted": token.decorations?.has("dim"),
                    "log-italic": token.decorations?.has("italic"),
                    "log-underline": token.decorations?.has("underline"),
                },
            });
        }

        parsed.value = items;
    },
    { immediate: true },
);

/**
 *
 * @param elt {ParseToken}
 */
function make_style(elt) {
    let ret = "";
    if (elt?.foreground) {
        ret += `color: ${elt.foreground};`;
    }
    if (elt?.background) {
        ret += `background-color: ${elt.background};`;
    }
    return ret;
}
</script>
<style scoped>
@reference "../style.css";

.log-line {
    text-indent: 5em hanging each-line;
    text-wrap: wrap;
    background-clip: content-box;
}
.log-item {
    @apply inline!;
    text-wrap: wrap;
}
.log-bold {
    @apply font-bold;
}
.log-muted {
    @apply text-muted-foreground;
}
.log-italic {
    @apply italic;
}
.log-underline {
    @apply underline;
}
.logdate {
    @apply text-muted-foreground bg-background;
    @apply shadow-sm rounded-sm ps-2 border-muted-foreground/20 border;
    &:has(+ .loglevel) {
        @apply border-e-0 rounded-e-none;
    }
    + .loglevel {
         @apply border-s-0 rounded-s-none;
    }
}
.loglevel-warn {
    @apply font-bold text-destructive;
    @apply not-dark:text-shadow-xs text-shadow-red-200;
}
.loglevel-trace {
    @apply font-bold text-muted-foreground;
    @apply not-dark:text-shadow-xs text-shadow-gray-300;
}
.loglevel {
    @apply bg-background;
    @apply shadow-sm rounded-sm px-1 border-muted-foreground/20 border;
}
.loglevel-debug {
    @apply font-bold text-cyan-700;
    @apply dark:text-cyan-500;
    @apply not-dark:text-shadow-sm text-shadow-gray-300;
}
.loglevel-error {
    @apply font-bold text-destructive;
    @apply not-dark:text-shadow-sm text-shadow-red-200;
}
.loglevel-info {
    @apply font-bold text-accent;
    @apply not-dark:text-shadow-sm text-shadow-gray-300;
}
</style>
<template>
    <div class="text-sm font-mono log-line fx-appear">
        <pre
            class="log-item"
            v-for="token in parsed"
            :style="make_style(token)"
            :class="token?.class"
            >{{ token.value }}</pre>
    </div>
</template>
