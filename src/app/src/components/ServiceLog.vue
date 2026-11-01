<script setup>
// SPDX-License-Identifier: Zlib
// SPDX-FileCopyrightText: 2026 Sylvain Fargier
// SPDX-FileContributor: Author: Sylvain Fargier <fargier.sylvain@gmail.com>

import { ref, watch } from "vue";
import { useWebSocket } from "@vueuse/core";
import { api_url } from "../Client";
import { addError } from "../store";
import LogLine from "./LogLine.vue";

/** @typedef {Array<{ line: string, idx: number }>} LogArray */

const log_lines = ref(/** @type {null|LogArray} */ (null));
const captured = ref(/** @type {null|LogArray} */ (null));

const MAX_LINES_MAX = 10000; // "infinite" log lines is not really infinite (preserve RAM)
const MAX_LINES_FETCH = 4096; // max lines to initially fetch
let line_idx = 0;
let last_line_complete = true;

const { id, lines, follow, max_lines, paused } = defineProps({
    id: { type: String },
    lines: { type: Number },
    follow: { type: Boolean, default: false },
    max_lines: { default: null },
    paused: { type: Boolean, default: false },
});
const emit = defineEmits(["update"]);
watch(
    () => max_lines,
  (max_lines) => {
    drop_lines(max_lines);
    if (max_lines !== null && log_lines.value?.length < max_lines) {
      /* re-fetch log  */
      close?.();
      close = log_stream();
    }
  },
);
watch(
    () => paused,
    (paused) => {
        console.log("toggling pause", paused);
        if (paused) {
            captured.value = [...(log_lines.value ?? [])];
        } else {
            captured.value = null;
        }
    },
);

/**
 *
 * @param max_lines {Number|null}
 */
function drop_lines(max_lines) {
    let max = max_lines ?? MAX_LINES_MAX;
    if (max !== null && log_lines.value.length > max) {
        log_lines.value.splice(0, log_lines.value.length - max);
    }
}

function log_stream() {
    let { close, send } = useWebSocket(api_url + "?raw=1", {
        autoReconnect: { delay: 1000 },
        onError(err) {
            addError(err?.message ?? "server connection closed");
        },
        onMessage: async (_, event) => {
            /** @type {String} */
            let data = await event.data.text();
            let new_lines = data.split("\n");
            if (log_lines.value === null) {
                log_lines.value = [];
            } else if (
                !last_line_complete &&
                new_lines.length > 0 &&
                log_lines.value.length > 0
            ) {
                /* logs are sent as byte streams, let's reconstruct lines */
                log_lines.value.at(-1).line += new_lines[0];
                new_lines.splice(0, 1);
            }

            if (data?.at(-1) === "\n") {
                new_lines.pop();
                last_line_complete = true;
            } else {
                last_line_complete = false;
            }

            for (let line of new_lines) {
                log_lines.value.push({ idx: line_idx++, line: line });
            }
            drop_lines(max_lines);
            emit("update");
        },
        onDisconnected() {
            log_lines.value = null;
        },
        onConnected() {
            send?.(
                JSON.stringify({
                    method: "log",
                    params: {
                        service: id,
                        lines: lines ?? MAX_LINES_FETCH,
                        follow,
                    },
                }),
            );
        },
    });
    return close;
}
let close = log_stream();
watch(
    () => id,
    (_) => {
        close?.();
        close = log_stream();
    },
);

function create_placeholder() {
    let ret = Array(max_lines ?? 3);
    for (let idx = 0; idx < ret.length; ++idx) {
        ret[idx] = { idx, line: null };
    }
    return ret;
}
</script>
<style scoped>
pre {
    text-indent: 3em hanging each-line;
    text-wrap: wrap;
    background-clip: content-box;
}
</style>
<template>
    <div class="p-1 w-full overflow-y-clip">
        <LogLine
            :key="idx"
            v-for="{ idx, line } in captured ??
            log_lines ??
            create_placeholder()"
            :line="line ?? ''"
            :class="{
                skel: line === null,
                'w-6/12': line === null && idx % 3 === 0,
                'w-5/12': line === null && idx % 3 === 1,
                'w-8/12': line === null && idx % 3 === 2,
            }"
        />
    </div>
</template>
