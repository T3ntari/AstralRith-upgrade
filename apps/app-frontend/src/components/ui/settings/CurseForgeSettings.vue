<script setup>
/**
 * CurseForge settings.
 *
 * No API key management exists here: the provider credential is application
 * infrastructure configured on the Rust side (environment / config file /
 * build-time), never a user-facing field.
 */
</script>

<template>
  <h2 class="m-0 text-lg font-extrabold text-contrast">CurseForge</h2>
  <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
    CurseForge content is available through the Discover page. No API key setup is needed — the
    launcher handles provider access automatically.
  </p>
</template>
