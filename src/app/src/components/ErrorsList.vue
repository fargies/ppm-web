<script setup>
import { errors } from "../store";
import ErrorsListEntry from "./ErrorsListEntry.vue";

</script>
<style scoped>
pre {
    text-indent: 4em hanging each-line;
}
</style>
<template>
    <div
        v-if="errors?.length > 0"
        title="application errors"
        class="fx-appear col-span-12 border-destructive border rounded shadow-sm"
    >
        <ErrorsListEntry
            :key="error?.id ?? idx"
            v-for="(error, idx) in errors"
            :index="idx" :error="error"
        />
    </div>
</template>
