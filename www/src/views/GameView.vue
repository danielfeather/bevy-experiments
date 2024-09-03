<template>
  <div class="container">
    <RouterLink to="/">Back</RouterLink>
    <h1>{{ $route.params.experiment }}</h1>
    <iframe class="bevy-game" :src="`/bevy/${$route.params.experiment}`" frameborder="0"></iframe>
  </div>
</template>

<style>
  .container {
    margin: 0 auto;
    max-width: 1200px;
  }
  .bevy-game {
    width: 100%;
    height: 100%;
    aspect-ratio: 1.7777;
  }
</style>
