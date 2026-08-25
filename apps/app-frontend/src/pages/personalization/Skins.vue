<template>
  <div class="flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="m-0 text-xl font-extrabold text-contrast">Skins</h1>
      <div class="flex gap-2">
        <Button @click="showLibrary = !showLibrary"><LibraryIcon /> {{ showLibrary ? 'Hide library' : 'My library' }}</Button>
        <Button color="primary" @click="handleImport"><UploadIcon /> Import PNG</Button>
      </div>
    </div>

    <!-- Browser: search + featured grid -->
    <div class="browser-section">
      <!-- Search -->
      <div class="flex flex-col gap-1.5">
        <h2 class="m-0 text-sm font-bold text-secondary">Find a player's skin</h2>
        <div class="flex gap-2">
          <div class="iconified-input flex-1 min-w-[200px]">
            <SearchIcon class="text-lg" />
            <input
              v-model="searchTerm"
              type="text"
              placeholder="Enter a Minecraft username or UUID (e.g. Notch, Tentari, Dream)..."
              class="h-9"
              @keydown.enter="searchPlayer"
            />
            <Button v-if="searchTerm" class="r-btn" @click="clearSearch"><XIcon /></Button>
          </div>
          <Button color="brand" :disabled="searching || !searchTerm.trim()" @click="searchPlayer">
            <template v-if="searching"><span class="animate-spin inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full" /></template>
            <template v-else><SearchIcon class="text-lg" /> Search</template>
          </Button>
        </div>
        <p class="m-0 text-xs text-secondary">Premium accounts only. Uses the free MCHeads API (mcheads.org) + Mojang profile lookup.</p>
      </div>

      <!-- Error -->
      <div v-if="searchError" class="err-box">
        <p class="m-0 text-sm">{{ searchError }}</p>
      </div>

      <!-- Search result card -->
      <div v-if="searchedPlayer" class="searched-card">
        <div class="player-card-inner" @click="openDetail(searchedPlayer)">
          <div class="player-preview">
            <img
              v-if="searchedPlayer.avatarUrl"
              :src="searchedPlayer.avatarUrl"
              class="player-render"
              alt=""
              loading="lazy"
            />
            <div v-else class="no-skin">No skin</div>
          </div>
          <div class="player-meta">
            <div class="player-name">{{ searchedPlayer.name }}</div>
            <div class="player-sub">UUID: {{ searchedPlayer.uuid }}</div>
            <div class="mt-2">
              <Button color="brand" class="!px-3 !py-1 !text-xs" @click.stop="openDetail(searchedPlayer)">View skin</Button>
            </div>
          </div>
        </div>
      </div>

      <!-- Featured browse grid -->
      <div class="flex flex-col gap-2">
        <h2 class="m-0 text-sm font-bold text-secondary">Browse featured players</h2>
        <div class="player-grid">
          <div
            v-for="p in featured"
            :key="p.name"
            class="player-card group cursor-pointer"
            @click="openDetail(p)"
          >
            <div class="player-preview">
              <img
                :src="p.avatarUrl"
                :alt="p.name"
                class="player-render"
                loading="lazy"
              />
            </div>
            <div class="player-name">{{ p.name }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Saved library -->
    <div v-if="showLibrary" class="library-section">
      <div class="flex flex-wrap items-center gap-2">
        <div class="iconified-input flex-1 min-w-[200px]">
          <SearchIcon class="text-lg" />
          <input v-model="store.searchQuery" type="text" placeholder="Search saved skins..." class="h-9" />
          <Button v-if="store.searchQuery" class="r-btn" @click="store.searchQuery = ''"><XIcon /></Button>
        </div>
        <DropdownSelect v-model="currentFilter" name="Filter" :options="filterOptions" :display-name="(o) => o.display" class="max-w-[10rem]">
          <FilterIcon class="mr-1" /><span class="text-sm text-secondary">{{ currentFilter.display }}</span>
        </DropdownSelect>
        <DropdownSelect v-model="currentSort" name="Sort" :options="sortOptions" :display-name="(o) => o.display" class="max-w-[10rem]">
          <span class="text-sm text-secondary">Sort: {{ currentSort.display }}</span>
        </DropdownSelect>
      </div>

      <div class="text-xs text-secondary">
        {{ store.filteredSkins.length }} saved skin{{ store.filteredSkins.length === 1 ? '' : 's' }}
        <span v-if="store.favoriteCount > 0"> · {{ store.favoriteCount }} favorite{{ store.favoriteCount === 1 ? '' : 's' }}</span>
      </div>

      <div v-if="store.loading" class="flex justify-center py-8">
        <div class="w-8 h-8 border-3 border-button-bg border-t-brand rounded-full animate-spin" />
      </div>
      <div v-else-if="store.filteredSkins.length === 0" class="flex flex-col items-center py-8 gap-2">
        <SparklesIcon class="w-10 h-10 text-secondary opacity-40" />
        <p class="m-0 text-secondary">No saved skins yet — browse a player above or import a PNG</p>
      </div>
      <div v-else class="skin-grid">
        <div v-for="skin in store.filteredSkins" :key="skin.id" class="skin-item group" @contextmenu="(e) => openContextMenu(e, skin)">
          <div class="skin-thumb">
            <img :src="skin.assetUrl" :alt="skin.name" class="skin-img" />
            <div v-if="skin.favorite" class="fav-badge"><HeartIcon class="w-3 h-3" fill="currentColor" /></div>
            <div class="skin-actions">
              <Button color="brand" class="!px-2 !py-0.5 !text-xs" @click.stop="applySkin(skin)">Apply</Button>
            </div>
          </div>
          <div class="skin-name">{{ skin.name }}</div>
          <div class="skin-meta">{{ skin.modelType }}<span v-if="skin.lastUsed"> · used {{ formatDate(skin.lastUsed) }}</span></div>
        </div>
      </div>
    </div>

    <!-- Player detail modal -->
    <ModalWrapper v-if="detail" :header="`${detail.name} — Skin`" :has-to-type="false" :on-hide="() => (detail = null)">
      <div class="player-detail">
        <div class="detail-preview">
          <img
            v-if="detail.avatarUrl"
            :src="detail.avatarUrl"
            class="detail-render"
            alt=""
          />
          <div v-else class="no-skin big">This player has no skin</div>
        </div>
        <div class="detail-info">
          <h3 class="m-0 text-lg font-bold text-contrast">{{ detail.name }}</h3>
          <p class="m-0 text-xs text-secondary">UUID: {{ detail.uuid || '—' }}</p>
          <div class="detail-actions flex gap-2 mt-3">
            <Button color="brand" :disabled="loadingBytes" @click="applyDetailSkin">
              <template v-if="loadingBytes"><span class="animate-spin inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full" /></template>
              <template v-else><CheckIcon /> Apply this skin</template>
            </Button>
            <Button :disabled="loadingBytes" @click="saveDetailSkin">
              <DownloadIcon /> Save to library
            </Button>
          </div>
          <p class="m-0 text-xs text-secondary mt-2">Downloads the skin from MCHeads and applies it to your account.</p>
        </div>
      </div>
    </ModalWrapper>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { Button, DropdownSelect } from '@modrinth/ui'
import {
  SearchIcon, XIcon, UploadIcon, HeartIcon, FilterIcon, DownloadIcon, CheckIcon, SparklesIcon, LibraryIcon,
} from '@modrinth/assets'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { useSkinsStore } from '@/store/skins'
import { handleError, useNotifications } from '@/store/notifications.js'
import {
  FEATURED_PLAYERS, avatarUrl, fetchSkinBytes, lookupMojangProfile,
} from '@/helpers/mcheads'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

const store = useSkinsStore()
const notifications = useNotifications()

// ---- Browse ----
const searchTerm = ref('')
const searching = ref(false)
const searchError = ref('')
const searchedPlayer = ref(null)
const detail = ref(null)
const loadingBytes = ref(false)
const showLibrary = ref(true)

const featured = computed(() =>
  FEATURED_PLAYERS.map((name) => ({
    name,
    avatarUrl: avatarUrl(name, 128, 'left'),
  })),
)

async function searchPlayer() {
  const term = searchTerm.value.trim()
  if (!term) return
  searching.value = true
  searchError.value = ''
  searchedPlayer.value = null
  try {
    const profile = await lookupMojangProfile(term)
    if (!profile) {
      searchError.value = `"${term}" is not a premium Minecraft account or doesn't exist.`
      return
    }
    searchedPlayer.value = {
      name: profile.name,
      uuid: profile.uuid,
      avatarUrl: avatarUrl(profile.name, 192, 'left'),
    }
  } catch (err) {
    searchError.value = `Lookup failed: ${err.message}`
  } finally {
    searching.value = false
  }
}

function clearSearch() {
  searchTerm.value = ''
  searchedPlayer.value = null
  searchError.value = ''
}

function openDetail(p) {
  detail.value = { ...p, avatarUrl: avatarUrl(p.name, 256, 'left') }
}

async function applyDetailSkin() {
  if (!detail.value) return
  loadingBytes.value = true
  try {
    const bytes = await fetchSkinBytes(detail.value.name)
    const record = await store.createSkin(detail.value.name, bytes, 'classic')
    if (record?.id) await store.markUsed(record.id)
    notifications.addNotification({ title: 'Skin applied', text: detail.value.name, type: 'success' })
    detail.value = null
  } catch (err) {
    handleError({ message: `Apply failed: ${err.message}` })
  } finally {
    loadingBytes.value = false
  }
}

async function saveDetailSkin() {
  if (!detail.value) return
  loadingBytes.value = true
  try {
    const bytes = await fetchSkinBytes(detail.value.name)
    await store.createSkin(detail.value.name, bytes, 'classic')
    notifications.addNotification({ title: 'Skin saved', text: detail.value.name, type: 'success' })
  } catch (err) {
    handleError({ message: `Save failed: ${err.message}` })
  } finally {
    loadingBytes.value = false
  }
}

// ---- Library ----
const filterOptions = [
  { value: 'all', display: 'All' },
  { value: 'favorites', display: 'Favorites' },
  { value: 'recently_used', display: 'Recently used' },
  { value: 'classic', display: 'Classic' },
  { value: 'slim', display: 'Slim' },
  { value: 'local', display: 'Local' },
  { value: 'imported', display: 'Imported' },
  { value: 'account', display: 'Account' },
]
const sortOptions = [
  { value: 'recently_added', display: 'Recently added' },
  { value: 'recently_used', display: 'Recently used' },
  { value: 'name', display: 'Name' },
  { value: 'custom', display: 'Custom' },
]
const currentFilter = computed({
  get: () => filterOptions.find((o) => o.value === store.filterMode) || filterOptions[0],
  set: (v) => (store.filterMode = v.value),
})
const currentSort = computed({
  get: () => sortOptions.find((o) => o.value === store.sortBy) || sortOptions[0],
  set: (v) => (store.sortBy = v.value),
})

async function handleImport() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      multiple: false,
      filters: [{ name: 'PNG', extensions: ['png'] }],
    })
    if (!file) return
    const resp = await tauriFetch(`asset://localhost/${file.replace(/^file:\/\//, '')}`)
    const bytes = await resp.arrayBuffer()
    const name = file.split('/').pop()?.replace(/\.png$/i, '') || 'Imported skin'
    await store.createSkin(name, new Uint8Array(bytes), 'classic')
    notifications.addNotification({ title: 'Skin imported', text: name, type: 'success' })
  } catch (err) {
    handleError({ message: `Import failed: ${err.message}` })
  }
}

async function applySkin(skin) {
  await store.markUsed(skin.id)
  notifications.addNotification({ title: 'Skin applied', text: skin.name, type: 'success' })
}

function openContextMenu(e, skin) {
  e.preventDefault()
  const action = confirm(`Skin: ${skin.name}\n\nOK = Apply\nCancel = Delete`)
  if (action) applySkin(skin)
  else if (confirm(`Delete "${skin.name}"?`)) store.remove(skin.id)
}

function formatDate(d) {
  if (!d) return ''
  const dt = new Date(d)
  return `${dt.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}`
}

onMounted(() => {
  store.refresh()
})
</script>

<style scoped lang="scss">
.browser-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background: var(--color-bg-raised);
  border-radius: 12px;
  padding: 1rem;
}

.player-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  gap: 0.6rem;
}

.player-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  background: var(--color-bg-base);
  border: 1px solid var(--color-button-bg);
  border-radius: 10px;
  padding: 0.5rem 0.25rem 0.4rem;
  transition: border-color 0.15s, transform 0.15s;

  &:hover {
    border-color: var(--color-brand);
    transform: translateY(-2px);
  }
}

.player-preview {
  width: 96px;
  height: 150px;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  background: radial-gradient(ellipse at 50% 100%, var(--color-brand-highlight), transparent 70%);
  border-radius: 8px;
  overflow: hidden;
}

.player-render {
  width: auto;
  height: 100%;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}

.player-name {
  font-weight: 700;
  font-size: 0.78rem;
  color: var(--color-contrast);
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.player-sub {
  font-size: 0.7rem;
  color: var(--color-secondary);
}

.searched-card {
  background: var(--color-bg-base);
  border: 1px solid var(--color-button-bg);
  border-radius: 10px;
  padding: 0.75rem;
}

.player-card-inner {
  display: flex;
  gap: 1rem;
  align-items: center;
  cursor: pointer;
}

.player-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.err-box {
  background: var(--color-red-bg, #fee2e2);
  border-left: 3px solid var(--color-red, #ef4444);
  padding: 0.5rem 0.75rem;
  border-radius: 4px;
  font-size: 0.8rem;
}

.library-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.skin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 0.75rem;
}

.skin-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skin-thumb {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  aspect-ratio: 1;
  background: var(--color-bg-raised);
}

.skin-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
}

.fav-badge {
  position: absolute;
  top: 4px;
  left: 4px;
  color: var(--color-red);
  background: rgba(0, 0, 0, 0.5);
  border-radius: 4px;
  padding: 2px;
}

.skin-actions {
  position: absolute;
  bottom: 4px;
  left: 4px;
  right: 4px;
  display: flex;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
}

.skin-item:hover .skin-actions {
  opacity: 1;
}

.skin-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-contrast);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skin-meta {
  font-size: 0.7rem;
  color: var(--color-secondary);
}

.player-detail {
  display: flex;
  gap: 1.5rem;
  padding: 0.5rem;
}

.detail-preview {
  flex-shrink: 0;
  width: 200px;
  height: 300px;
  background: radial-gradient(ellipse at 50% 100%, var(--color-brand-highlight), transparent 70%);
  border-radius: 12px;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  overflow: hidden;
}

.detail-render {
  width: auto;
  height: 100%;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}

.detail-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.no-skin {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: 0.7rem;
  color: var(--color-secondary);

  &.big { font-size: 0.9rem; }
}

@media (max-width: 640px) {
  .player-detail {
    flex-direction: column;
  }
}
</style>
