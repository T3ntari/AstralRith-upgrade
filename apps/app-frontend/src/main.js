import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'
import { createPlugin } from '@vintl/vintl/plugin'

const VIntlPlugin = createPlugin({
  controllerOpts: {
    defaultLocale: 'en-US',
    locale: 'en-US',
    locales: [
      {
        tag: 'en-US',
        meta: {
          displayName: 'American English',
        },
      },
    ],
  },
  globalMixin: true,
  injectInto: [],
})

const pinia = createPinia()

// Restore the user's last content-source choice (Modrinth vs CurseForge)
document.documentElement.classList.toggle('cf-mode', localStorage.getItem('ar.cf_mode') === 'true')

let app = createApp(App)

app.use(router)
app.use(pinia)
app.use(FloatingVue, {
  themes: {
    'ribbit-popout': {
      $extend: 'dropdown',
      placement: 'bottom-end',
      instantMove: true,
      distance: 8,
    },
  },
})
app.use(VIntlPlugin)

app.mount('#app')
