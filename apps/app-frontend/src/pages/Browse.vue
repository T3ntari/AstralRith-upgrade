<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import type { Ref } from 'vue'
import { SearchIcon, XIcon, ClipboardCopyIcon, GlobeIcon, ExternalIcon } from '@modrinth/assets'
import type { Category, GameVersion, Platform, ProjectType, SortType, Tags } from '@modrinth/ui'
import {
  SearchFilterControl,
  SearchSidebarFilter,
  Button,
  Checkbox,
  DropdownSelect,
  LoadingIndicator,
  Pagination,
  useSearch,
} from '@modrinth/ui'
import { handleError } from '@/store/state'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import type { LocationQuery } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile.js'
import { get_search_results } from '@/helpers/cache.js'
import NavTabs from '@/components/ui/NavTabs.vue'
import type Instance from '@/components/ui/Instance.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import ContentSourceToggle from '@/components/ui/ContentSourceToggle.vue'
import CfSearchCard from '@/components/ui/CfSearchCard.vue'
import CfInstallModal from '@/components/ui/CfInstallModal.vue'
import {
  getCfMode,
  setCfMode,
  getCfApiKey,
  setCfApiKey,
  searchMods,
  getCategories,
  getGameVersions,
  getModloaders,
  getFileDownloadUrl,
  loaderIdForPlatform,
  classIdForProjectType,
  CF_SORT_FIELDS,
  CF_LOADER_NAMES,
} from '@/helpers/curseforge.js'
import { invoke } from '@tauri-apps/api/core'

const { formatMessage } = useVIntl()

const router = useRouter()
const route = useRoute()

const projectTypes = computed(() => {
  return [route.params.projectType as ProjectType]
})

const [categories, loaders, availableGameVersions] = await Promise.all([
  get_categories().catch(handleError).then(ref),
  get_loaders().catch(handleError).then(ref),
  get_game_versions().catch(handleError).then(ref),
])

const tags: Ref<Tags> = computed(() => ({
  gameVersions: availableGameVersions.value as GameVersion[],
  loaders: loaders.value as Platform[],
  categories: categories.value as Category[],
}))

type Instance = {
  game_version: string
  loader: string
  path: string
  install_stage: string
  icon_path?: string
  name: string
}

type InstanceProject = {
  metadata: {
    project_id: string
  }
}

const instance: Ref<Instance | null> = ref(null)
const instanceProjects: Ref<InstanceProject[] | null> = ref(null)
const instanceHideInstalled = ref(false)
const newlyInstalled = ref([])

const PERSISTENT_QUERY_PARAMS = ['i', 'ai']

async function updateInstanceContext() {
  if (route.query.i) {
    ;[instance.value, instanceProjects.value] = await Promise.all([
      getInstance(route.query.i).catch(handleError),
      getInstanceProjects(route.query.i).catch(handleError),
    ])
    newlyInstalled.value = []

    if (isCurseForge.value && instance.value) {
      cfGameVersion.value = {
        name: instance.value.game_version,
        display: instance.value.game_version,
      }
      cfLoader.value = {
        name: loaderIdForPlatform(instance.value.loader),
        display: 'Instance loader',
      }
    }
  }

  if (route.query.ai && !(projectTypes.value.length === 1 && projectTypes.value[0] === 'modpack')) {
    instanceHideInstalled.value = route.query.ai === 'true'
  }

  if (instance.value && instance.value.path !== route.query.i && route.path.startsWith('/browse')) {
    instance.value = null
    instanceHideInstalled.value = false
  }
}

const instanceFilters = computed(() => {
  const filters = []

  if (instance.value) {
    const gameVersion = instance.value.game_version
    if (gameVersion) {
      filters.push({
        type: 'game_version',
        option: gameVersion,
      })
    }

    const platform = instance.value.loader

    const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

    if (platform && projectTypes.value.includes('mod') && supportedModLoaders.includes(platform)) {
      filters.push({
        type: 'mod_loader',
        option: platform,
      })
    }

    if (instanceHideInstalled.value && instanceProjects.value) {
      const installedMods = Object.values(instanceProjects.value)
        .filter((x) => x.metadata)
        .map((x) => x.metadata.project_id)

      installedMods.push(...newlyInstalled.value)

      installedMods
        ?.map((x) => ({
          type: 'project_id',
          option: `project_id:${x}`,
          negative: true,
        }))
        .forEach((x) => filters.push(x))
    }
  }

  return filters
})

const {
  // Selections
  query,
  currentSortType,
  currentFilters,
  toggledGroups,
  maxResults,
  currentPage,
  overriddenProvidedFilterTypes,

  // Lists
  filters,
  sortTypes,

  // Computed
  requestParams,

  // Functions
  createPageParams,
} = useSearch(projectTypes, tags, instanceFilters)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

// ----- CurseForge mode -----
const isCurseForge = ref(getCfMode())
const cfApiKey = ref(getCfApiKey())
const cfKeyInput = ref('')
const showCfKeyPrompt = computed(() => isCurseForge.value && !cfApiKey.value)

const cfCategories = ref([])
const cfGameVersions = ref([])
const cfLoaderList = ref([])
const cfCategory = ref({ name: '', display: 'Any category' })
const cfGameVersion = ref({ name: '', display: 'Any version' })
const cfLoader = ref({ name: 0, display: 'Any loader' })
const cfSortField = ref({ name: 2, display: 'Popularity' })
const cfResults = shallowRef([])
const cfTotalHits = ref(0)
const cfLoading = ref(false)
const cfPage = ref(0)
const cfPageSize = 20
const cfInstalling = ref('')
const cfInstalled = ref([])

const cfSortOptions = Object.entries(CF_SORT_FIELDS).map(([name, display]) => ({
  name: Number(name),
  display,
}))

const cfLoaderOptions = computed(() => [
  { name: 0, display: 'Any loader' },
  ...cfLoaderList.value
    .map((loader) => ({
      name: loader.modLoaderType,
      display:
        CF_LOADER_NAMES[loader.modLoaderType] ?? loader.name ?? `Loader ${loader.modLoaderType}`,
    }))
    .filter((opt, index, arr) => arr.findIndex((o) => o.name === opt.name) === index),
])

const cfCategoryOptions = computed(() => [
  { name: '', display: 'Any category' },
  ...cfCategories.value.map((cat) => ({ name: cat.id, display: cat.name })),
])

const cfGameVersionOptions = computed(() => [
  { name: '', display: 'Any version' },
  ...cfGameVersions.value.map((v) => ({ name: v, display: v })),
])

const cfPageCount = computed(() => Math.max(1, Math.ceil(cfTotalHits.value / cfPageSize)))

watch(isCurseForge, (enabled) => {
  setCfMode(enabled)
  if (enabled) {
    loadCfData()
      .then(() => {
        cfPage.value = 0
        return cfSearch()
      })
      .catch(handleError)
  }
})

watch(
  () => route.params.projectType,
  () => {
    if (!isCurseForge.value) return
    cfPage.value = 0
    loadCfData()
      .then(() => cfSearch())
      .catch(handleError)
  },
)

watch([cfSortField, cfGameVersion, cfLoader, cfCategory, cfPage], () => {
  if (!isCurseForge.value) return
  cfSearch()
})

watch(
  () => query.value,
  () => {
    if (!isCurseForge.value) return
    cfPage.value = 0
  },
)

// Load CurseForge data on first render if the mode is already enabled
if (isCurseForge.value && cfApiKey.value) {
  await loadCfData()
  await cfSearch()
}

await updateInstanceContext()

watch(route, () => {
  updateInstanceContext()
})

async function loadCfData() {
  const [cats, versions, loaders] = await Promise.all([
    getCategories(classIdForProjectType(projectType.value)).catch(() => []),
    getGameVersions().catch(() => []),
    getModloaders().catch(() => []),
  ])
  cfCategories.value = cats ?? []
  cfGameVersions.value = versions ?? []
  cfLoaderList.value = loaders ?? []
}

async function cfSearch() {
  if (!cfApiKey.value) {
    cfResults.value = []
    cfTotalHits.value = 0
    return
  }
  cfLoading.value = true
  try {
    const { results, totalCount } = await searchMods({
      projectType: projectType.value,
      query: query.value,
      page: cfPage.value,
      pageSize: cfPageSize,
      sortField: cfSortField.value?.name ?? 2,
      gameVersion: cfGameVersion.value?.name ?? '',
      modLoaderType: cfLoader.value?.name ?? 0,
      categoryId: cfCategory.value?.name || undefined,
    })
    cfResults.value = results
    cfTotalHits.value = totalCount
  } catch (err) {
    console.error('CurseForge search failed', err)
    cfResults.value = []
    cfTotalHits.value = 0
  } finally {
    cfLoading.value = false
  }
}

async function saveCfKey() {
  const trimmed = cfKeyInput.value.trim()
  if (!trimmed) return
  setCfApiKey(trimmed)
  cfApiKey.value = trimmed
  cfKeyInput.value = ''
  await loadCfData()
  await cfSearch()
}

const cfPendingMod = ref(null)

async function handleCfInstall(mod) {
  if (!instance.value || cfInstalling.value) return
  cfPendingMod.value = mod
  cfInstallModal.value?.show()
}

async function cfInstallSelectedFile(file) {
  const mod = cfPendingMod.value
  if (!mod || !instance.value || cfInstalling.value) return
  const cfId = mod.id
  cfInstalling.value = cfId
  try {
    if (projectType.value === 'modpack') {
      const url = file.downloadUrl ?? (await getFileDownloadUrl(cfId, file.id))
      await invoke('plugin:cf|cf_install_modpack', {
        profilePath: instance.value.path,
        packUrl: url,
        packName: mod.name,
        apiKey: cfApiKey.value,
      })
    } else {
      const url = file.downloadUrl ?? (await getFileDownloadUrl(cfId, file.id))
      await invoke('plugin:cf|cf_install_mod', {
        profilePath: instance.value.path,
        fileUrl: url,
        fileName: file.fileName,
        displayName: mod.name,
        fileLength: file.fileLength,
      })
    }
    cfInstalled.value.push(String(cfId))
  } catch (err) {
    console.error('Failed to install CurseForge content', err)
    handleError({ message: `Failed to install ${mod.name}: ${err.message}` })
  } finally {
    cfInstalling.value = ''
    cfPendingMod.value = null
  }
}

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: route.query })

const loading = ref(true)

const projectType = ref(route.params.projectType)

watch(projectType, () => {
  loading.value = true
})

type SearchResult = {
  project_id: string
}

type SearchResults = {
  total_hits: number
  limit: number
  hits: SearchResult[]
}

const results: Ref<SearchResults | null> = shallowRef(null)
const pageCount = computed(() =>
  results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

watch(requestParams, () => {
  if (!route.params.projectType) return
  refreshSearch()
})

async function refreshSearch() {
  if (isCurseForge.value) return
  let rawResults = await get_search_results(requestParams.value)
  if (!rawResults) {
    rawResults = {
      result: {
        hits: [],
        total_hits: 0,
        limit: 1,
      },
    }
  }
  if (instance.value) {
    for (const val of rawResults.result.hits) {
      val.installed =
        newlyInstalled.value.includes(val.project_id) ||
        Object.values(instanceProjects.value).some(
          (x) => x.metadata && x.metadata.project_id === val.project_id,
        )
    }
  }
  results.value = rawResults.result

  const persistentParams: LocationQuery = {}

  for (const [key, value] of Object.entries(route.query)) {
    if (PERSISTENT_QUERY_PARAMS.includes(key)) {
      persistentParams[key] = value
    }
  }

  if (instanceHideInstalled.value) {
    persistentParams.ai = 'true'
  } else {
    delete persistentParams.ai
  }

  const params = {
    ...persistentParams,
    ...createPageParams(),
  }

  breadcrumbs.setContext({
    name: 'Discover content',
    link: `/browse/${projectType.value}`,
    query: params,
  })
  await router.replace({ path: route.path, query: params })
  loading.value = false
}

async function setPage(newPageNumber: number) {
  currentPage.value = newPageNumber

  await onSearchChangeToTop()
}

const searchWrapper: Ref<HTMLElement | null> = ref(null)

async function onSearchChangeToTop() {
  await nextTick()

  window.scrollTo({ top: 0, behavior: 'smooth' })
}

function clearSearch() {
  query.value = ''
}

watch(
  () => route.params.projectType,
  async (newType) => {
    // Check if the newType is not the same as the current value
    if (!newType || newType === projectType.value) return

    projectType.value = newType

    currentSortType.value = { display: 'Relevance', name: 'relevance' }
    query.value = ''
  },
)

const selectableProjectTypes = computed(() => {
  let dataPacks = false,
    mods = false,
    modpacks = false

  if (instance.value) {
    if (
      availableGameVersions.value.findIndex((x) => x.version === instance.value.game_version) <=
      availableGameVersions.value.findIndex((x) => x.version === '1.13')
    ) {
      dataPacks = true
    }

    if (instance.value.loader !== 'vanilla') {
      mods = true
    }
  } else {
    dataPacks = true
    mods = true
    modpacks = true
  }

  const params: LocationQuery = {}

  if (route.query.i) {
    params.i = route.query.i
  }
  if (route.query.ai) {
    params.ai = route.query.ai
  }

  const links = [
    { label: 'Modpacks', href: `/browse/modpack`, shown: modpacks },
    { label: 'Mods', href: `/browse/mod`, shown: mods },
    { label: 'Resource Packs', href: `/browse/resourcepack` },
    { label: 'Data Packs', href: `/browse/datapack`, shown: dataPacks },
    { label: 'Shaders', href: `/browse/shader` },
  ]

  if (params) {
    return links.map((link) => {
      return {
        ...link,
        href: {
          path: link.href,
          query: params,
        },
      }
    })
  }

  return links
})

const messages = defineMessages({
  gameVersionProvidedByInstance: {
    id: 'search.filter.locked.instance-game-version.title',
    defaultMessage: 'Game version is provided by the instance',
  },
  modLoaderProvidedByInstance: {
    id: 'search.filter.locked.instance-loader.title',
    defaultMessage: 'Loader is provided by the instance',
  },
  providedByInstance: {
    id: 'search.filter.locked.instance',
    defaultMessage: 'Provided by the instance',
  },
  syncFilterButton: {
    id: 'search.filter.locked.instance.sync',
    defaultMessage: 'Sync with instance',
  },
})

const options = ref(null)
const handleRightClick = (event, result) => {
  options.value.showMenu(event, result, [
    {
      name: 'open_link',
    },
    {
      name: 'copy_link',
    },
  ])
}
const handleOptionsClick = (args) => {
  switch (args.option) {
    case 'open_link':
      openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
      break
    case 'copy_link':
      navigator.clipboard.writeText(
        `https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
      )
      break
  }
}

await refreshSearch()
</script>

<template>
  <Teleport v-if="filters && !isCurseForge" to="#sidebar-teleport-target">
    <div
      v-if="instance"
      class="border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
    >
      <Checkbox
        v-model="instanceHideInstalled"
        label="Hide installed content"
        class="filter-checkbox"
        @update:model-value="onSearchChangeToTop()"
        @click.prevent.stop
      />
    </div>
    <SearchSidebarFilter
      v-for="filter in filters.filter((f) => f.display !== 'none')"
      :key="`filter-${filter.id}`"
      v-model:selected-filters="currentFilters"
      v-model:toggled-groups="toggledGroups"
      v-model:overridden-provided-filter-types="overriddenProvidedFilterTypes"
      :provided-filters="instanceFilters"
      :filter-type="filter"
      class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
      button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
      content-class="mb-3"
      inner-panel-class="ml-2 mr-3"
      :open-by-default="
        filter.id.startsWith('category') || filter.id === 'environment' || filter.id === 'license'
      "
    >
      <template #header>
        <h3 class="text-base m-0">{{ filter.formatted_name }}</h3>
      </template>
      <template #locked-game_version>
        {{ formatMessage(messages.gameVersionProvidedByInstance) }}
      </template>
      <template #locked-mod_loader>
        {{ formatMessage(messages.modLoaderProvidedByInstance) }}
      </template>
      <template #sync-button> {{ formatMessage(messages.syncFilterButton) }} </template>
    </SearchSidebarFilter>
  </Teleport>
  <div ref="searchWrapper" class="flex flex-col gap-3 p-6">
    <template v-if="instance">
      <InstanceIndicator :instance="instance" />
      <h1 class="m-0 mb-1 text-xl">Install content to instance</h1>
    </template>
    <NavTabs :links="selectableProjectTypes" />
    <div class="flex justify-between items-center gap-3 flex-wrap">
      <div v-if="isCurseForge && !showCfKeyPrompt" class="flex gap-2 flex-wrap">
        <DropdownSelect
          v-slot="{ selected }"
          v-model="cfSortField"
          name="Sort by"
          :options="cfSortOptions"
          :display-name="(option) => option?.display"
          class="max-w-[12rem]"
        >
          <span class="font-semibold text-primary">Sort: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
        <DropdownSelect
          v-slot="{ selected }"
          v-model="cfGameVersion"
          name="Game version"
          :options="cfGameVersionOptions"
          :display-name="(option) => option?.display"
          class="max-w-[11rem]"
        >
          <span class="font-semibold text-primary">Version: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
        <DropdownSelect
          v-slot="{ selected }"
          v-model="cfLoader"
          name="Loader"
          :options="cfLoaderOptions"
          :display-name="(option) => option?.display"
          class="max-w-[10rem]"
        >
          <span class="font-semibold text-primary">Loader: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
        <DropdownSelect
          v-slot="{ selected }"
          v-model="cfCategory"
          name="Category"
          :options="cfCategoryOptions"
          :display-name="(option) => option?.display"
          class="max-w-[12rem]"
        >
          <span class="font-semibold text-primary">Category: </span>
          <span class="font-semibold text-secondary">{{ selected }}</span>
        </DropdownSelect>
      </div>
      <ContentSourceToggle v-model="isCurseForge" class="ml-auto" />
    </div>
    <div class="iconified-input">
      <SearchIcon aria-hidden="true" class="text-lg" />
      <input
        v-model="query"
        class="h-12 card-shadow"
        autocomplete="off"
        spellcheck="false"
        type="text"
        :placeholder="
          isCurseForge ? `Search ${projectType}s on CurseForge...` : `Search ${projectType}s...`
        "
      />
      <Button v-if="query" class="r-btn" @click="() => clearSearch()">
        <XIcon />
      </Button>
    </div>
    <div v-if="!isCurseForge" class="flex gap-2">
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentSortType"
        class="max-w-[16rem]"
        name="Sort by"
        :options="sortTypes as any"
        :display-name="(option: SortType | undefined) => option?.display"
      >
        <span class="font-semibold text-primary">Sort by: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="maxResults"
        name="Max results"
        :options="[5, 10, 15, 20, 50, 100]"
        class="max-w-[9rem]"
      >
        <span class="font-semibold text-primary">View: </span>
        <span class="font-semibold text-secondary">{{ selected }}</span>
      </DropdownSelect>
      <Pagination :page="currentPage" :count="pageCount" class="ml-auto" @switch-page="setPage" />
    </div>
    <div v-if="!isCurseForge">
      <SearchFilterControl
        v-model:selected-filters="currentFilters"
        :filters="filters.filter((f) => f.display !== 'none')"
        :provided-filters="instanceFilters"
        :overridden-provided-filter-types="overriddenProvidedFilterTypes"
        :provided-message="messages.providedByInstance"
      />
    </div>
    <div class="search" style="contain: layout style;">
      <section v-if="isCurseForge && showCfKeyPrompt" class="offline">
        <div class="cf-key-card card-shadow p-6 bg-bg-raised rounded-xl max-w-[32rem] text-left">
          <h2 class="text-xl font-bold m-0 mb-2 text-contrast">CurseForge mode needs an API key</h2>
          <p class="text-secondary m-0 mb-4">
            CurseForge requires an API key (free, from
            <a class="text-primary underline" href="https://console.curseforge.com" target="_blank"
              >console.curseforge.com</a
            >) to search and install content. Your key is stored only on this device and can be
            changed later in Settings.
          </p>
          <input
            v-model="cfKeyInput"
            type="password"
            class="h-12 w-full card-shadow mb-4 px-4 rounded-xl"
            placeholder="Paste your CurseForge API key..."
            @keyup.enter="saveCfKey"
          />
          <div class="flex gap-2">
            <Button color="primary" @click="saveCfKey">Enable CurseForge</Button>
            <Button @click="isCurseForge = false">Back to Modrinth</Button>
          </div>
        </div>
      </section>
      <section v-else-if="isCurseForge && cfLoading" class="offline">
        <LoadingIndicator />
      </section>
      <section v-else-if="isCurseForge && cfResults.length === 0" class="offline">
        No CurseForge results found. Try a different search or filter.
      </section>
      <section
        v-else-if="isCurseForge"
        class="project-list display-mode--list instance-results"
        role="list"
      >
        <CfSearchCard
          v-for="mod in cfResults"
          v-memo="[mod.id, cfInstalled.includes(String(mod.id))]"
          :key="mod.id"
          :mod="mod"
          :instance="instance"
          :modpack="projectType === 'modpack'"
          :installed="cfInstalled.includes(String(mod.id))"
          :installing="cfInstalling === mod.id"
          @install="handleCfInstall(mod)"
        />
        <CfInstallModal
          ref="cfInstallModal"
          :mod="cfPendingMod"
          :instance="instance"
          :modpack="projectType === 'modpack'"
          @install="cfInstallSelectedFile"
        />
      </section>
      <section v-else-if="loading" class="offline">
        <LoadingIndicator />
      </section>
      <section v-else-if="offline && results.total_hits === 0" class="offline">
        You are currently offline. Connect to the internet to browse Modrinth!
      </section>
      <section v-else class="project-list display-mode--list instance-results" role="list">
        <SearchCard
          v-for="result in results.hits"
          v-memo="[result.project_id, result.installed]"
          :key="result?.project_id"
          :project="result"
          :instance="instance"
          :categories="[
            ...categories.filter(
              (cat) =>
                result?.display_categories.includes(cat.name) && cat.project_type === projectType,
            ),
            ...loaders.filter(
              (loader) =>
                result?.display_categories.includes(loader.name) &&
                loader.supported_project_types?.includes(projectType),
            ),
          ]"
          :installed="result.installed || newlyInstalled.includes(result.project_id)"
          @install="
            (id) => {
              newlyInstalled.push(id)
            }
          "
          @contextmenu.prevent.stop="(event) => handleRightClick(event, result)"
        />
        <ContextMenu ref="options" @option-clicked="handleOptionsClick">
          <template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
          <template #copy_link> <ClipboardCopyIcon /> Copy link </template>
        </ContextMenu>
      </section>
      <div class="flex justify-end">
        <pagination
          :page="isCurseForge ? cfPage + 1 : currentPage"
          :count="isCurseForge ? cfPageCount : pageCount"
          class="pagination-after"
          @switch-page="
            (page) => {
              if (isCurseForge) {
                cfPage = page - 1
                onSearchChangeToTop()
              } else {
                setPage(page)
              }
            }
          "
        />
      </div>
    </div>
  </div>
</template>
