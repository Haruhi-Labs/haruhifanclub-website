<script setup>
// Figure 节点的 Vue NodeView：展示图片 + 可就地编辑的图注输入。
import { NodeViewWrapper } from '@tiptap/vue-3'

const props = defineProps({
  node: { type: Object, required: true },
  updateAttributes: { type: Function, required: true },
  selected: { type: Boolean, default: false },
})

function onCaption(e) {
  props.updateAttributes({ caption: e.target.value })
}
</script>

<template>
  <NodeViewWrapper class="rte-figure" :class="{ 'is-selected': selected }">
    <img class="rte-figure__img" :src="node.attrs.src" :alt="node.attrs.alt" draggable="false" />
    <input
      class="rte-figure__cap"
      :value="node.attrs.caption"
      placeholder="添加图注（可留空）"
      maxlength="200"
      @input="onCaption"
      @mousedown.stop
      @keydown.stop
    />
  </NodeViewWrapper>
</template>

<style scoped>
.rte-figure {
  margin: 1.2em 0;
  text-align: center;
}
.rte-figure.is-selected {
  outline: 2px solid var(--sos-accent);
  outline-offset: 3px;
  border-radius: var(--sos-radius-sm);
}
.rte-figure__img {
  max-width: 100%;
  height: auto;
  border-radius: var(--sos-radius-md);
  display: block;
  margin: 0 auto;
}
.rte-figure__cap {
  display: block;
  width: 100%;
  margin-top: 8px;
  border: none;
  background: transparent;
  text-align: center;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-tertiary);
  outline: none;
}
.rte-figure__cap::placeholder {
  color: var(--sos-text-tertiary);
  opacity: 0.6;
}
</style>
