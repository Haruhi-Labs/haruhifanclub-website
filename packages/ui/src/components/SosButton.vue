<script setup lang="ts">
import { computed } from 'vue'
import { safeUrl } from '../internal/safe-url'

type ButtonElement = 'button' | 'a'
type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
type ButtonSize = 'sm' | 'md' | 'lg'
type ButtonType = 'button' | 'submit' | 'reset'

const props = withDefaults(
  defineProps<{
    as?: ButtonElement
    variant?: ButtonVariant
    size?: ButtonSize
    type?: ButtonType
    href?: string
    disabled?: boolean
    loading?: boolean
  }>(),
  {
    as: 'button',
    variant: 'primary',
    size: 'md',
    type: 'button',
    href: undefined,
    disabled: false,
    loading: false,
  }
)

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const isUnavailable = computed(() => props.disabled || props.loading)
// 仅在链接形态且可用时输出 href，并经协议白名单过滤，拦截 javascript: 等注入
const safeHref = computed(() =>
  props.as === 'a' && !isUnavailable.value ? safeUrl(props.href) : undefined
)
const classes = computed(() => [
  'sos-button',
  `sos-button--${props.variant}`,
  props.size !== 'md' ? `sos-button--${props.size}` : undefined,
])

function onClick(event: MouseEvent) {
  if (isUnavailable.value) {
    event.preventDefault()
    event.stopPropagation()
    return
  }

  emit('click', event)
}
</script>

<template>
  <component
    :is="props.as"
    :class="classes"
    :type="props.as === 'button' ? props.type : undefined"
    :href="safeHref"
    :disabled="props.as === 'button' ? isUnavailable : undefined"
    :aria-disabled="props.as === 'a' && isUnavailable ? 'true' : undefined"
    :aria-busy="props.loading ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-if="!props.loading" />
    <slot v-else name="loading"> 处理中 </slot>
  </component>
</template>
