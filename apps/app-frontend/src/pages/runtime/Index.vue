<script setup>
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'
import NavTabs from '@/components/ui/NavTabs.vue'

const breadcrumbs = useBreadcrumbs()
const route = useRoute()

breadcrumbs.setRootContext({ name: 'Runtime', link: '/runtime' })

const isOverview = computed(() => route.path === '/runtime' || route.path === '/runtime/')

const tabs = [
  { label: 'Clients', href: '/runtime' },
  { label: 'Bots', href: '/runtime/bots' },
  { label: 'Modules', href: '/runtime/modules' },
]
</script>

<template>
  <div class="p-6 flex flex-col gap-4 h-full overflow-auto">
    <h1 class="m-0 text-2xl font-extrabold text-contrast">Runtime</h1>
    <NavTabs :links="tabs" />
    <RouterView />
  </div>
</template>
