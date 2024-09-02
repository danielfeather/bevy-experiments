<template>
  <iframe class="bevy-game" :src="`/bevy/${$route.params.experiment}`" frameborder="0"></iframe>
</template>

<style>
  .bevy-game {
    width: 100%;
    height: 100%;
  }
</style>
