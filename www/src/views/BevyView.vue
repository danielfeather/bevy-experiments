<script setup lang="ts">
import { onMounted } from 'vue';
import { useRoute } from 'vue-router'

const route = useRoute();

const experiments: Record<string, () => Promise<any>> = {
    'elastic-box': () => import('experiments/elastic-box'),
    'resize': () => import('experiments/resize')
} as const


onMounted(async () => {
    const experiment: string = Array.isArray(route.params.experiment) ? route.params.experiment[0] : route.params.experiment;

    const wasm = experiments[experiment] ?? undefined;

    if (!wasm) {
        console.error(`Couldn't find experiment with name ${experiment}`)
    }

    const game = await wasm();

    game.start()
})

</script>

<template>
    <canvas id="bevy"></canvas>
</template>
  
<style>

</style>
