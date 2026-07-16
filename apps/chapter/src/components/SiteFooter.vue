<script setup>
import { haruhiLogoUrl } from '@haruhi/auth-ui'
import { SosSiteFooter, defaultSiteFooterGroups } from '@haruhi/ui'

const bottomLinks = [
  {
    label: '皖ICP备2025089290号-1',
    href: 'https://beian.miit.gov.cn/#/Integrated/index',
  },
]
</script>

<template>
  <SosSiteFooter
    :logo-src="haruhiLogoUrl"
    brand-subtitle="地方支部网络 · Haruhifanclub"
    :groups="defaultSiteFooterGroups"
    :bottom-links="bottomLinks"
  />
</template>
