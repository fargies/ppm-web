<script setup>
import { Collapsible, Button } from ".";
import { X } from "@lucide/vue";
import { computed, ref, watch } from "vue";
import { errors, addError } from "../store";

const { error, index } = defineProps(["error", "index"]);

const firstLine = ref("");
const content = ref("");

watch(
    () => error,
    () => {
        let split = (error?.message ?? error ?? "")
            .toString()
            .trim()
            .split("\n");
        firstLine.value = split[0];
        content.value = split.splice(1).join("\n");
    },
    { immediate: true },
);
</script>
<style scoped>
@reference "../style.css";

pre {
    @apply text-destructive text-wrap overflow-y-hidden wrap-break-word fx-appear;
}
</style>

<template>
    <div
        class="flex fx-appear col-span-12 items-center gap-1 hover:bg-foreground/5"
    >
        <Collapsible v-if="content" class="border-none! grow">
            <template #summary>
                <span class="font-mono text-xs text-muted-foreground"
                    >[{{ error?.id }}]</span
                >
                <pre
                    class="grow"
                ><span class="font-bold">ERROR: </span>{{ firstLine }}</pre>
            </template>

            <pre class="grow wrap-break-word overflow-y-hidden text-wrap font-mono text-xs">{{
                content
            }}</pre>
        </Collapsible>
        <div class="flex items-center p-1 gap-1" v-else>
            <span class="font-mono text-xs text-muted-foreground"
                >[{{ error?.id }}]</span
            >
            <pre
                class="grow"
            ><span class="font-bold">ERROR: </span>{{ firstLine }}</pre>
        </div>
        <div class="ml-auto mb-auto mt-1 pe-1">
            <Button
                title="dismiss error"
                label="dismiss error"
                class="cursor-pointer border-none!"
                @click="errors.splice(index, 1)"
            >
                <X />
            </Button>
        </div>
    </div>
</template>
