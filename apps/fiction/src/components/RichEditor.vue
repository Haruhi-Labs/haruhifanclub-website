<script setup>
import { watch, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import Link from '@tiptap/extension-link'
import Image from '@tiptap/extension-image'
import Placeholder from '@tiptap/extension-placeholder'
import { uploadCover, coverUrl } from '@/api'

const props = defineProps({
  modelValue: { type: String, default: '' },
  placeholder: { type: String, default: '在此写下你的故事……' },
})
const emit = defineEmits(['update:modelValue'])

const editor = useEditor({
  content: props.modelValue,
  extensions: [
    StarterKit.configure({ heading: { levels: [2, 3, 4] } }),
    Underline,
    Link.configure({ openOnClick: false, autolink: true, HTMLAttributes: { rel: 'noopener noreferrer nofollow' } }),
    Image.configure({ inline: false }),
    Placeholder.configure({ placeholder: () => props.placeholder }),
  ],
  onUpdate: ({ editor }) => emit('update:modelValue', editor.getHTML()),
})

// 外部（如切换章节 / 载入草稿）改动才 setContent，避免与 onUpdate 抖动
watch(
  () => props.modelValue,
  (v) => {
    if (editor.value && v !== editor.value.getHTML()) {
      editor.value.commands.setContent(v || '', false)
    }
  },
)

onBeforeUnmount(() => editor.value?.destroy())

function toggleLink() {
  const prev = editor.value.getAttributes('link').href
  const url = window.prompt('链接地址', prev || 'https://')
  if (url === null) return
  if (url === '') {
    editor.value.chain().focus().unsetLink().run()
    return
  }
  editor.value.chain().focus().extendMarkRange('link').setLink({ href: url }).run()
}

async function onPickImage(e) {
  const file = e.target.files?.[0]
  if (!file) return
  try {
    const r = await uploadCover(file)
    editor.value.chain().focus().setImage({ src: coverUrl(r.path) }).run()
  } catch {
    /* 忽略上传失败 */
  }
  e.target.value = ''
}
</script>

<template>
  <div class="rte">
    <div v-if="editor" class="rte__bar">
      <button type="button" :class="{ on: editor.isActive('bold') }" title="加粗" @click="editor.chain().focus().toggleBold().run()"><b>B</b></button>
      <button type="button" :class="{ on: editor.isActive('italic') }" title="斜体" @click="editor.chain().focus().toggleItalic().run()"><i>I</i></button>
      <button type="button" :class="{ on: editor.isActive('underline') }" title="下划线" @click="editor.chain().focus().toggleUnderline().run()"><u>U</u></button>
      <button type="button" :class="{ on: editor.isActive('strike') }" title="删除线" @click="editor.chain().focus().toggleStrike().run()"><s>S</s></button>
      <span class="rte__sep"></span>
      <button type="button" :class="{ on: editor.isActive('heading', { level: 2 }) }" title="标题" @click="editor.chain().focus().toggleHeading({ level: 2 }).run()">H2</button>
      <button type="button" :class="{ on: editor.isActive('heading', { level: 3 }) }" title="小标题" @click="editor.chain().focus().toggleHeading({ level: 3 }).run()">H3</button>
      <button type="button" :class="{ on: editor.isActive('blockquote') }" title="引用" @click="editor.chain().focus().toggleBlockquote().run()">❝</button>
      <button type="button" :class="{ on: editor.isActive('bulletList') }" title="无序列表" @click="editor.chain().focus().toggleBulletList().run()">•</button>
      <button type="button" :class="{ on: editor.isActive('orderedList') }" title="有序列表" @click="editor.chain().focus().toggleOrderedList().run()">1.</button>
      <button type="button" title="分景线" @click="editor.chain().focus().setHorizontalRule().run()">— —</button>
      <span class="rte__sep"></span>
      <button type="button" :class="{ on: editor.isActive('link') }" title="链接" @click="toggleLink">🔗</button>
      <button type="button" title="插图" @click="$refs.img.click()">🖼</button>
      <input ref="img" type="file" accept="image/*" hidden @change="onPickImage" />
      <span class="rte__sep"></span>
      <button type="button" title="撤销" :disabled="!editor.can().undo()" @click="editor.chain().focus().undo().run()">↶</button>
      <button type="button" title="重做" :disabled="!editor.can().redo()" @click="editor.chain().focus().redo().run()">↷</button>
    </div>
    <EditorContent :editor="editor" class="rte__body" />
  </div>
</template>

<style scoped>
.rte {
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-md);
  background: var(--sos-bg-surface);
  overflow: hidden;
}
.rte__bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--sos-border-subtle);
  background: var(--sos-bg-subtle);
  position: sticky;
  top: 0;
  z-index: 5;
}
.rte__bar button {
  min-width: 30px;
  height: 30px;
  border: none;
  background: transparent;
  border-radius: var(--sos-radius-sm);
  cursor: pointer;
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
}
.rte__bar button:hover {
  background: var(--sos-bg-muted);
}
.rte__bar button.on {
  background: var(--sos-accent);
  color: #fff;
}
.rte__bar button:disabled {
  opacity: 0.35;
  cursor: default;
}
.rte__sep {
  width: 1px;
  height: 18px;
  background: var(--sos-border-default);
  margin: 0 4px;
}
.rte__body {
  padding: var(--sos-space-5);
  min-height: 420px;
  font-family: var(--sos-font-reading);
  font-size: 17px;
  line-height: 1.9;
  color: var(--sos-text-primary);
}
.rte__body :deep(.ProseMirror) {
  outline: none;
  min-height: 380px;
}
.rte__body :deep(.ProseMirror p) {
  margin: 0 0 1em;
}
.rte__body :deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  color: var(--sos-text-tertiary);
  float: left;
  height: 0;
  pointer-events: none;
}
.rte__body :deep(.ProseMirror h2),
.rte__body :deep(.ProseMirror h3) {
  font-family: var(--sos-font-reading);
  margin: 1.2em 0 0.6em;
}
.rte__body :deep(.ProseMirror blockquote) {
  border-left: 3px solid var(--sos-accent);
  padding-left: 1em;
  color: var(--sos-text-secondary);
  margin: 1em 0;
}
.rte__body :deep(.ProseMirror hr) {
  border: none;
  border-top: 1px solid var(--sos-border-default);
  margin: 1.5em 0;
}
.rte__body :deep(.ProseMirror img) {
  max-width: 100%;
  border-radius: var(--sos-radius-sm);
}
</style>
