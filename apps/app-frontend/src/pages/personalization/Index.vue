<script setup>
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'
import { useSkinsStore } from '@/store/skins.js'

const breadcrumbs = useBreadcrumbs()
const skinsStore = useSkinsStore()

breadcrumbs.setRootContext({ name: 'Personalization', link: '/personalization' })

onMounted(async () => {
  await skinsStore.refresh().catch(() => {})
})
</script>

<template>
  <div class="p-6 flex flex-col gap-4 h-full overflow-auto">
    <RouterView />
  </div>
</template>
