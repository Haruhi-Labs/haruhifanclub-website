<script setup>
import { SosEmptyState, SosButton } from '@haruhi/ui'
import { useRouter } from 'vue-router'
const router = useRouter()
</script>

<template>
  <div class="fiction-page fiction-page--narrow">
    <SosEmptyState title="这里什么都没有" copy="页面走丢在了封闭空间里。">
      <template #actions>
        <SosButton variant="primary" @click="router.push('/')">回到首页</SosButton>
        <SosButton variant="secondary" @click="router.push('/library')">去书库看看</SosButton>
      </template>
    </SosEmptyState>
  </div>
</template>
