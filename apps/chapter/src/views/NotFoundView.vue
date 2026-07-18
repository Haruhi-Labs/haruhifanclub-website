<script setup>
import { RouterLink } from 'vue-router'
import { SosEmptyState, SosPage } from '@haruhi/ui'
</script>
<template>
  <SosPage>
    <SosEmptyState title="页面不存在" copy="这个地址可能已经移动。">
      <template #actions>
        <RouterLink to="/" class="sos-button sos-button--primary"> 返回支部目录 </RouterLink>
      </template>
    </SosEmptyState>
  </SosPage>
</template>
