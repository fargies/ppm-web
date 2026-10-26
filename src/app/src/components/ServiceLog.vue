<script setup>
import { ref, watch } from "vue";
import { useWebSocket } from "@vueuse/core";
import { api_url } from "../Client";
import { addError } from "../store";

/** @typedef {Array<{ line: string, idx: number }>} LogArray */

const log_lines = ref(/** @type {null|LogArray} */ (null));
const captured = ref(/** @type {null|LogArray} */ (null));
let line_idx = 1000;
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
        if (max_lines !== null) {
            log_lines.value.splice(0, log_lines.value.length - max_lines);
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
            if (max_lines !== null && log_lines.value.length > max_lines) {
                log_lines.value.splice(0, log_lines.value.length - max_lines);
            }
            emit("update");
        },
        onDisconnected() {
            log_lines.value = null;
        },
        onConnected() {
            send?.(
                JSON.stringify({
                    method: "log",
                    params: { service: id, lines, follow },
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
        <pre
            :key="idx"
            v-for="{ idx, line } in captured ??
            log_lines ??
            create_placeholder()"
            class="py-0.5 font-mono text-xs skel-anim wrap-break-word overflow-y-hidden fx-appear"
            :class="{
                skel: line === null,
                'w-6/12': line === null && idx % 3 === 0,
                'w-5/12': line === null && idx % 3 === 1,
                'w-8/12': line === null && idx % 3 === 2,
            }"
            >{{ (line ?? "placeholder") || " " }}</pre>
    </div>
</template>
