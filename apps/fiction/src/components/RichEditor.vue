<script setup>
import { ref, watch, onBeforeUnmount } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import Link from '@tiptap/extension-link'
import Image from '@tiptap/extension-image'
import Highlight from '@tiptap/extension-highlight'
import Typography from '@tiptap/extension-typography'
import CharacterCount from '@tiptap/extension-character-count'
import Placeholder from '@tiptap/extension-placeholder'
import { useToast } from '@haruhi/ui'
import { Figure } from '@/lib/tiptap/figure'
import { uploadCover, coverUrl } from '@/api'

const toast = useToast()

const props = defineProps({
  modelValue: { type: String, default: '' },
  placeholder: { type: String, default: '在此写下你的故事……' },
})
const emit = defineEmits(['update:modelValue'])

const chars = ref(0)
const uploading = ref(false)
const imgInput = ref(null)

function refreshCount(ed) {
  chars.value = ed?.storage?.characterCount?.characters?.() ?? 0
}

const editor = useEditor({
  content: props.modelValue,
  extensions: [
    StarterKit.configure({ heading: { levels: [2, 3, 4] } }),
    Underline,
    Link.configure({
      openOnClick: false,
      autolink: true,
      HTMLAttributes: { rel: 'noopener noreferrer nofollow' },
    }),
    Image.configure({ inline: false }), // 兼容历史 <img>
    Figure, // 新配图统一用带图注的 figure
    Highlight,
    Typography,
    CharacterCount,
    Placeholder.configure({ placeholder: () => props.placeholder }),
  ],
  editorProps: {
    handlePaste: (_view, event) => {
      const file = Array.from(event.clipboardData?.files || []).find((f) =>
        f.type.startsWith('image/'),
      )
      if (file) {
        uploadAndInsert(file)
        return true
      }
      return false
    },
    handleDrop: (_view, event) => {
      const file = Array.from(event.dataTransfer?.files || []).find((f) =>
        f.type.startsWith('image/'),
      )
      if (file) {
        event.preventDefault()
        uploadAndInsert(file)
        return true
      }
      return false
    },
  },
  onCreate: ({ editor }) => refreshCount(editor),
  onUpdate: ({ editor }) => {
    emit('update:modelValue', editor.getHTML())
    refreshCount(editor)
  },
})

// 外部（切换章节 / 载入草稿）改动才 setContent，避免与 onUpdate 抖动
watch(
  () => props.modelValue,
  (v) => {
    if (editor.value && v !== editor.value.getHTML()) {
      editor.value.commands.setContent(v || '', false)
      refreshCount(editor.value)
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

async function uploadAndInsert(file) {
  uploading.value = true
  try {
    const r = await uploadCover(file)
    editor.value.chain().focus().setFigure({ src: coverUrl(r.path), alt: file.name || '' }).run()
  } catch (e) {
    toast.danger(e.message || '插图上传失败')
  } finally {
    uploading.value = false
  }
}

function onPickImage(e) {
  const file = e.target.files?.[0]
  if (file) uploadAndInsert(file)
  e.target.value = ''
}
</script>

<template>
  <div class="rte">
    <div v-if="editor" class="rte__bar">
      <div class="rte__group">
        <button type="button" :class="{ on: editor.isActive('heading', { level: 2 }) }" title="标题" @click="editor.chain().focus().toggleHeading({ level: 2 }).run()">H2</button>
        <button type="button" :class="{ on: editor.isActive('heading', { level: 3 }) }" title="小标题" @click="editor.chain().focus().toggleHeading({ level: 3 }).run()">H3</button>
        <button type="button" :class="{ on: editor.isActive('heading', { level: 4 }) }" title="次级标题" @click="editor.chain().focus().toggleHeading({ level: 4 }).run()">H4</button>
      </div>
      <span class="rte__sep"></span>
      <div class="rte__group">
        <button type="button" :class="{ on: editor.isActive('bold') }" title="加粗" @click="editor.chain().focus().toggleBold().run()"><b>B</b></button>
        <button type="button" :class="{ on: editor.isActive('italic') }" title="斜体" @click="editor.chain().focus().toggleItalic().run()"><i>I</i></button>
        <button type="button" :class="{ on: editor.isActive('underline') }" title="下划线" @click="editor.chain().focus().toggleUnderline().run()"><u>U</u></button>
        <button type="button" :class="{ on: editor.isActive('strike') }" title="删除线" @click="editor.chain().focus().toggleStrike().run()"><s>S</s></button>
        <button type="button" class="rte__hl" :class="{ on: editor.isActive('highlight') }" title="高亮" @click="editor.chain().focus().toggleHighlight().run()">A</button>
      </div>
      <span class="rte__sep"></span>
      <div class="rte__group">
        <button type="button" :class="{ on: editor.isActive('blockquote') }" title="引用" @click="editor.chain().focus().toggleBlockquote().run()">❝</button>
        <button type="button" :class="{ on: editor.isActive('bulletList') }" title="无序列表" @click="editor.chain().focus().toggleBulletList().run()">•</button>
        <button type="button" :class="{ on: editor.isActive('orderedList') }" title="有序列表" @click="editor.chain().focus().toggleOrderedList().run()">1.</button>
        <button type="button" :class="{ on: editor.isActive('codeBlock') }" title="代码块" @click="editor.chain().focus().toggleCodeBlock().run()">{ }</button>
        <button type="button" title="场景分隔" @click="editor.chain().focus().setHorizontalRule().run()">— —</button>
      </div>
      <span class="rte__sep"></span>
      <div class="rte__group">
        <button type="button" :class="{ on: editor.isActive('link') }" title="链接" @click="toggleLink">🔗</button>
        <button type="button" title="插图" :disabled="uploading" @click="imgInput.click()">🖼</button>
        <input ref="imgInput" type="file" accept="image/*" hidden @change="onPickImage" />
      </div>
      <span class="rte__sep"></span>
      <div class="rte__group">
        <button type="button" title="撤销" :disabled="!editor.can().undo()" @click="editor.chain().focus().undo().run()">↶</button>
        <button type="button" title="重做" :disabled="!editor.can().redo()" @click="editor.chain().focus().redo().run()">↷</button>
      </div>
      <span class="rte__count">{{ uploading ? '上传中…' : `${chars} 字` }}</span>
    </div>
    <EditorContent :editor="editor" class="rte__body" />
  </div>
</template>

<style scoped>
.rte {
  border: 1px solid var(--sos-border-default);
  border-radius: var(--sos-radius-lg);
  background: var(--sos-bg-surface);
  overflow: hidden;
}
/* 焦点由外层容器统一用边框表达，内部编辑区不再出环，
   避免与外框叠成双重边框 / 内发光（设计系统 .sos-scope :focus-visible 出的 box-shadow 环） */
.rte:focus-within {
  border-color: var(--sos-focus, var(--sos-accent));
}
.rte__bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--sos-border-subtle);
  background: var(--sos-bg-subtle);
  position: sticky;
  top: 0;
  z-index: 5;
}
.rte__group {
  display: flex;
  align-items: center;
  gap: 2px;
}
.rte__bar button {
  min-width: 30px;
  height: 30px;
  padding: 0 7px;
  border: none;
  background: transparent;
  border-radius: var(--sos-radius-sm);
  cursor: pointer;
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  line-height: 1;
}
.rte__bar button:hover {
  background: var(--sos-bg-muted);
  color: var(--sos-text-primary);
}
.rte__bar button.on {
  background: var(--sos-accent);
  color: #fff;
}
.rte__bar button:disabled {
  opacity: 0.35;
  cursor: default;
}
.rte__hl {
  background: linear-gradient(transparent 62%, var(--sos-signal, #ffe08a) 62%);
}
.rte__hl.on {
  background: var(--sos-accent);
}
.rte__sep {
  width: 1px;
  height: 18px;
  background: var(--sos-border-default);
  margin: 0 4px;
}
.rte__count {
  margin-left: auto;
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
  font-variant-numeric: var(--sos-numeric-tabular);
  padding-inline: 4px;
}
.rte__body {
  padding: var(--sos-space-6) var(--sos-space-7);
  min-height: 460px;
  font-family: var(--sos-font-reading);
  font-size: 17px;
  line-height: 1.9;
  color: var(--sos-text-primary);
}
.rte__body :deep(.ProseMirror),
.rte__body :deep(.ProseMirror:focus),
.rte__body :deep(.ProseMirror:focus-visible) {
  outline: none;
  box-shadow: none; /* 抵消 .sos-scope 全局焦点环，改由 .rte:focus-within 出单层边框 */
}
.rte__body :deep(.ProseMirror) {
  min-height: 400px;
}
.rte__body :deep(.ProseMirror > * + *) {
  margin-top: 0.9em;
}
.rte__body :deep(.ProseMirror p) {
  margin: 0;
}
.rte__body :deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  color: var(--sos-text-tertiary);
  float: left;
  height: 0;
  pointer-events: none;
}
.rte__body :deep(.ProseMirror h2) {
  font-size: 1.5em;
  font-weight: 700;
  margin: 1.4em 0 0.5em;
}
.rte__body :deep(.ProseMirror h3) {
  font-size: 1.25em;
  font-weight: 700;
  margin: 1.3em 0 0.5em;
}
.rte__body :deep(.ProseMirror h4) {
  font-size: 1.1em;
  font-weight: 700;
  margin: 1.2em 0 0.5em;
}
.rte__body :deep(.ProseMirror blockquote) {
  border-left: 3px solid var(--sos-accent);
  padding-left: 1em;
  color: var(--sos-text-secondary);
  margin: 1em 0;
}
.rte__body :deep(.ProseMirror mark) {
  background: var(--sos-signal, #ffe08a);
  border-radius: 2px;
  padding: 0 2px;
  color: inherit;
}
.rte__body :deep(.ProseMirror pre) {
  background: var(--sos-bg-muted);
  border-radius: var(--sos-radius-md);
  padding: var(--sos-space-4);
  font-family: var(--sos-font-mono);
  font-size: 0.9em;
  line-height: 1.6;
  overflow-x: auto;
}
.rte__body :deep(.ProseMirror pre code) {
  background: none;
  padding: 0;
  font-size: inherit;
}
.rte__body :deep(.ProseMirror code) {
  background: var(--sos-bg-muted);
  border-radius: 3px;
  padding: 1px 5px;
  font-family: var(--sos-font-mono);
  font-size: 0.88em;
}
.rte__body :deep(.ProseMirror hr) {
  border: none;
  text-align: center;
  margin: 1.6em 0;
}
.rte__body :deep(.ProseMirror hr)::before {
  content: '❋';
  color: var(--sos-text-tertiary);
}
.rte__body :deep(.ProseMirror img) {
  max-width: 100%;
  border-radius: var(--sos-radius-md);
}
</style>
