<script setup lang="ts">
import { useAboutStore } from '@/stores/about'
import { Coffee, Github, Globe, Heart, X } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'
import * as Neutralino from '@neutralinojs/lib'

const aboutStore = useAboutStore()
const version = ref('0.1.0')

onMounted(async () => {
  if (window.NL_PORT) {
    try {
      const config = await Neutralino.app.getConfig()
      version.value = config.version
    } catch (err) {
      console.warn('Failed to get app config:', err)
    }
  }
})

const openLink = (url: string) => {
  if (window.NL_PORT) {
    Neutralino.os.open(url)
  } else {
    window.open(url, '_blank')
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="aboutStore.isOpen"
        class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
        @click.self="aboutStore.close">
        
        <div class="bg-surface border border-border rounded-2xl shadow-2xl w-full max-w-[380px] overflow-hidden animate-in zoom-in duration-200">
          <!-- Branded Header -->
          <div class="relative h-32 bg-gradient-to-br from-primary to-primary-hover flex items-center justify-center overflow-hidden">
            <!-- Decorative background elements -->
            <div class="absolute inset-0 opacity-10">
              <svg class="w-full h-full" viewBox="0 0 100 100" preserveAspectRatio="none">
                <path d="M0 0 L100 100 M100 0 L0 100" stroke="white" stroke-width="0.5" />
              </svg>
            </div>
            
            <div class="relative flex flex-col items-center gap-2">
               <div class="w-14 h-14 bg-white/10 backdrop-blur-md rounded-2xl border border-white/20 flex items-center justify-center shadow-xl">
                 <svg class="w-8 h-8 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                   <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                   <line x1="3" y1="9" x2="21" y2="9" />
                   <line x1="9" y1="21" x2="9" y2="9" />
                 </svg>
               </div>
               <h2 class="text-xl font-bold text-white tracking-tight">Table View</h2>
            </div>

            <button @click="aboutStore.close"
              class="absolute top-4 right-4 p-1.5 bg-white/10 hover:bg-white/20 text-white rounded-full transition-colors backdrop-blur-md">
              <X :size="18" />
            </button>
          </div>

          <!-- Content -->
          <div class="p-6 space-y-6">
            <div class="text-center space-y-2.5">
              <div class="inline-flex px-2.5 py-0.5 rounded-full bg-primary/10 text-primary text-[11px] font-bold uppercase tracking-wider">
                Version {{ version }}
              </div>
              <p class="text-text-secondary text-[13px] leading-relaxed px-2">
                A modern, high-performance database management tool designed for developers who value speed, simplicity, and a beautiful user interface.
              </p>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <button @click="openLink('https://github.com/vantoan1511/table-view')"
                class="flex items-center justify-center gap-2 px-4 py-2.5 bg-surface-lighter hover:bg-hover border border-border rounded-xl text-sm font-medium text-text-primary transition-all">
                <Github :size="16" />
                GitHub
              </button>
              <button @click="openLink('https://github.com/vantoan1511')"
                class="flex items-center justify-center gap-2 px-4 py-2.5 bg-surface-lighter hover:bg-hover border border-border rounded-xl text-sm font-medium text-text-primary transition-all">
                <Globe :size="16" />
                Toan Nguyen
              </button>
            </div>

            <div class="h-px bg-border/50" />

            <div class="flex flex-col gap-3">
              <button @click="openLink('https://www.buymeacoffee.com/vantoan1511')"
                class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-[#FFDD00] hover:bg-[#FFDD00]/90 text-black rounded-xl text-sm font-bold transition-all shadow-sm active:scale-[0.98]">
                <Coffee :size="18" fill="currentColor" />
                Buy me a coffee
              </button>
              <div class="flex items-center justify-center gap-1.5 text-[11px] text-text-tertiary">
                Made with <Heart :size="10" class="text-danger fill-danger" /> by Toan Nguyen
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
