<script setup lang="ts">
/**
 * SplitPanel — a generic, resizable two-pane vertical split container.
 *
 * Props:
 *   split        – Whether the split is active (shows both panels + divider)
 *   splitRatio   – Percentage of height for the top panel (20–80)
 *   bottomTitle  – Title shown in the bottom panel header
 *
 * Emits:
 *   update:splitRatio – When the user drags the divider
 *   moveToTop         – When the user clicks the "move to top" button
 *   minimize          – When the user clicks the "minimize" button
 *   close             – When the user clicks the "close" button
 *
 * Slots:
 *   top     – Content for the top panel (always rendered)
 *   bottom  – Content for the bottom panel (only when split is true)
 */
import { ArrowUp, Minus, X } from 'lucide-vue-next'
import { ref } from 'vue'

const props = withDefaults(defineProps<{
  split?: boolean
  splitRatio?: number
  bottomTitle?: string
}>(), {
  split: false,
  splitRatio: 50,
  bottomTitle: '',
})

const emit = defineEmits<{
  'update:splitRatio': [value: number]
  moveToTop: []
  minimize: []
  close: []
}>()

// ─── Resizable Divider ────────────────────────────────────────────────────────
const containerRef = ref<HTMLElement | null>(null)

const onDividerMouseDown = (e: MouseEvent) => {
  e.preventDefault()
  const container = containerRef.value
  if (!container) return

  const startY = e.clientY
  const startRatio = props.splitRatio
  const containerRect = container.getBoundingClientRect()

  const onMouseMove = (ev: MouseEvent) => {
    const deltaY = ev.clientY - startY
    const deltaPct = (deltaY / containerRect.height) * 100
    emit('update:splitRatio', Math.max(20, Math.min(80, startRatio + deltaPct)))
  }

  const onMouseUp = () => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}
</script>

<template>
  <div ref="containerRef" class="flex flex-col flex-1 min-w-0 min-h-0">
    <template v-if="split">
      <!-- Top Panel -->
      <div class="flex flex-col min-w-0 min-h-0 overflow-hidden" :style="{ flex: `0 0 ${splitRatio}%` }">
        <slot name="top" />
      </div>

      <!-- Resize Divider -->
      <div
        class="h-[5px] bg-border hover:bg-primary/40 cursor-row-resize transition-colors shrink-0 flex items-center justify-center group"
        @mousedown="onDividerMouseDown"
      >
        <div class="w-8 h-[3px] rounded bg-text-tertiary/30 group-hover:bg-primary/60 transition-colors"></div>
      </div>

      <!-- Bottom Panel -->
      <div class="flex flex-col min-w-0 min-h-0 flex-1 overflow-hidden">
        <!-- Bottom panel header -->
        <div class="flex items-center gap-2 px-3 py-1 bg-muted border-b border-border shrink-0">
          <span class="text-[12px] font-medium text-text-primary truncate">{{ bottomTitle }}</span>
          <div class="flex items-center gap-0.5 ml-auto">
            <button
              class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-primary hover:bg-hover transition-colors cursor-pointer"
              title="Move to top"
              @click="emit('moveToTop')"
            >
              <ArrowUp :size="12" />
            </button>
            <button
              class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-text-primary hover:bg-hover transition-colors cursor-pointer"
              title="Minimize"
              @click="emit('minimize')"
            >
              <Minus :size="12" />
            </button>
            <button
              class="flex items-center justify-center w-5 h-5 rounded text-text-tertiary hover:text-danger hover:bg-danger-light transition-colors cursor-pointer"
              title="Close panel"
              @click="emit('close')"
            >
              <X :size="12" />
            </button>
          </div>
        </div>
        <slot name="bottom" />
      </div>
    </template>

    <!-- Single view (no split) -->
    <template v-else>
      <slot name="top" />
    </template>
  </div>
</template>
