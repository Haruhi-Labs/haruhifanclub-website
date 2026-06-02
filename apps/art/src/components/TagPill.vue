<template>
  <button class="tag" @click="$emit('click', tag)" :title="`筛选：${tag}`">
    #{{ tag }}
  </button>
</template>

<script setup>
defineProps({ tag: { type: String, required: true } })
defineEmits(['click'])
</script>

<style scoped>
.tag{
  border:0;
  background: transparent;
  cursor:pointer;
  font-weight:900;
  font-size:12px;
  color: var(--blue);
  padding:4px 6px;
  border-radius:999px;
}
.tag:hover{
  background: rgba(109, 126, 155, 0.939);
}
</style>
