<script setup lang="ts">
import { useToast } from '../useToast'

const { toasts, dismiss } = useToast()
</script>

<template>
  <Teleport to="body">
    <div class="sos-toast-region sos-scope" role="region" aria-live="polite">
      <article
        v-for="toast in toasts"
        :key="toast.id"
        class="sos-toast"
        :class="{
          'sos-toast--success': toast.tone === 'success',
          'sos-toast--danger': toast.tone === 'danger',
        }"
      >
        <span aria-hidden="true">
          <template v-if="toast.tone === 'success'">✓</template>
          <template v-else-if="toast.tone === 'danger'">!</template>
          <template v-else>i</template>
        </span>
        <div>
          <p v-if="toast.title" class="sos-toast__title">{{ toast.title }}</p>
          <p class="sos-toast__copy">{{ toast.message }}</p>
        </div>
        <button
          type="button"
          class="sos-button sos-button--ghost sos-icon-button sos-button--sm"
          aria-label="关闭"
          @click="dismiss(toast.id)"
        >
          ×
        </button>
      </article>
    </div>
  </Teleport>
</template>
