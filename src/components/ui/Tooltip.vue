<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';

const props = defineProps<{
  text: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
  preferredPosition?: 'top' | 'bottom' | 'left' | 'right';
}>();

const triggerRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const show = ref(false);
const coords = ref({ top: 0, left: 0 });
const currentActivePosition = ref<'top' | 'bottom' | 'left' | 'right'>('bottom');

let targetEl: HTMLElement | null = null;

const fits = (pos: 'top' | 'bottom' | 'left' | 'right', rect: DOMRect) => {
  let w = 160;
  let h = 36;

  if (tooltipRef.value) {
    const tooltipRect = tooltipRef.value.getBoundingClientRect();
    if (tooltipRect.width > 0) w = tooltipRect.width;
    if (tooltipRect.height > 0) h = tooltipRect.height;
  }

  const offset = 8;

  switch (pos) {
    case 'top':
      return (
        rect.top >= h + offset &&
        rect.left + rect.width / 2 - w / 2 >= 0 &&
        rect.left + rect.width / 2 + w / 2 <= window.innerWidth
      );
    case 'bottom':
      return (
        rect.bottom + h + offset <= window.innerHeight &&
        rect.left + rect.width / 2 - w / 2 >= 0 &&
        rect.left + rect.width / 2 + w / 2 <= window.innerWidth
      );
    case 'left':
      return (
        rect.left >= w + offset &&
        rect.top + rect.height / 2 - h / 2 >= 0 &&
        rect.top + rect.height / 2 + h / 2 <= window.innerHeight
      );
    case 'right':
      return (
        rect.right + w + offset <= window.innerWidth &&
        rect.top + rect.height / 2 - h / 2 >= 0 &&
        rect.top + rect.height / 2 + h / 2 <= window.innerHeight
      );
  }
};

const determineActivePosition = (rect: DOMRect): 'top' | 'bottom' | 'left' | 'right' => {
  if (props.position !== undefined) {
    return props.position;
  }

  const preference = props.preferredPosition;
  if (preference === undefined) {
    return 'bottom';
  }

  const order: ('top' | 'bottom' | 'left' | 'right')[] = [];
  if (preference === 'top') {
    order.push('top', 'bottom', 'right', 'left');
  } else if (preference === 'bottom') {
    order.push('bottom', 'top', 'right', 'left');
  } else if (preference === 'left') {
    order.push('left', 'right', 'top', 'bottom');
  } else {
    order.push('right', 'left', 'top', 'bottom');
  }

  for (const pos of order) {
    if (fits(pos, rect)) {
      return pos;
    }
  }

  return preference;
};

const updatePosition = () => {
  const el =
    triggerRef.value?.tagName === 'DIV' ? triggerRef.value : triggerRef.value?.parentElement;
  if (!el) return;

  const rect = el.getBoundingClientRect();

  const pos = determineActivePosition(rect);
  currentActivePosition.value = pos;

  let x = 0;
  let y = 0;

  switch (pos) {
    case 'top':
      x = rect.left + rect.width / 2;
      y = rect.top;
      break;
    case 'left':
      x = rect.left;
      y = rect.top + rect.height / 2;
      break;
    case 'right':
      x = rect.right;
      y = rect.top + rect.height / 2;
      break;
    case 'bottom':
    default:
      x = rect.left + rect.width / 2;
      y = rect.bottom;
      break;
  }

  coords.value = { top: y, left: x };
};

let animationFrameId: number | null = null;

const handleMouseEnter = () => {
  show.value = true;
  updatePosition();
  nextTick(() => {
    updatePosition();
  });
};

const handleMouseLeave = () => {
  show.value = false;
  if (animationFrameId !== null) {
    window.cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
};

const handleScrollOrResize = () => {
  if (show.value && animationFrameId === null) {
    animationFrameId = window.requestAnimationFrame(() => {
      updatePosition();
      animationFrameId = null;
    });
  }
};

onMounted(() => {
  if (!triggerRef.value) return;

  targetEl = triggerRef.value.tagName === 'DIV' ? triggerRef.value : triggerRef.value.parentElement;

  if (targetEl) {
    targetEl.addEventListener('mouseenter', handleMouseEnter);
    targetEl.addEventListener('mouseleave', handleMouseLeave);
  }

  window.addEventListener('scroll', handleScrollOrResize, true);
  window.addEventListener('resize', handleScrollOrResize);
});

onUnmounted(() => {
  if (targetEl) {
    targetEl.removeEventListener('mouseenter', handleMouseEnter);
    targetEl.removeEventListener('mouseleave', handleMouseLeave);
  }

  window.removeEventListener('scroll', handleScrollOrResize, true);
  window.removeEventListener('resize', handleScrollOrResize);

  if (animationFrameId !== null) {
    window.cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
});

const tooltipStyle = computed(() => {
  let transform = '';
  switch (currentActivePosition.value) {
    case 'top':
      transform = 'translate(-50%, -100%) translateY(-8px)';
      break;
    case 'left':
      transform = 'translate(-100%, -50%) translateX(-8px)';
      break;
    case 'right':
      transform = 'translate(0, -50%) translateX(8px)';
      break;
    case 'bottom':
    default:
      transform = 'translate(-50%, 0) translateY(8px)';
      break;
  }

  return {
    top: `${coords.value.top}px`,
    left: `${coords.value.left}px`,
    transform
  };
});

const arrowClasses = computed(() => {
  const base = 'bg-surface border-border absolute h-2 w-2 rotate-45 border-solid';

  let arrowPosClasses = '';
  switch (currentActivePosition.value) {
    case 'top':
      arrowPosClasses = '-bottom-1 left-1/2 -translate-x-1/2 border-r border-b';
      break;
    case 'left':
      arrowPosClasses = '-right-1 top-1/2 -translate-y-1/2 border-t border-r';
      break;
    case 'right':
      arrowPosClasses = '-left-1 top-1/2 -translate-y-1/2 border-b border-l';
      break;
    case 'bottom':
    default:
      arrowPosClasses = '-top-1 left-1/2 -translate-x-1/2 border-t border-l';
      break;
  }

  return `${base} ${arrowPosClasses}`;
});
</script>

<template>
  <div v-if="$slots.default" ref="triggerRef" class="inline-block">
    <slot />
  </div>
  <span v-else ref="triggerRef" style="display: none"></span>

  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-100 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-75 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="show && text"
        ref="tooltipRef"
        class="bg-surface border-border text-text-primary pointer-events-none fixed z-[9999] max-w-64 rounded border px-2 py-1 text-[11px] font-medium break-words whitespace-normal shadow-xl"
        :style="tooltipStyle"
      >
        {{ text }}
        <div :class="arrowClasses"></div>
      </div>
    </Transition>
  </Teleport>
</template>
