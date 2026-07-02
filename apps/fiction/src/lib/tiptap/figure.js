// 配图块：图片 + 可选图注（对齐 harublog editor 的 Figure —— atom 块，attrs src/alt/caption，
// 序列化为 <figure><img><figcaption>）。图注通过 Vue NodeView 就地编辑。
import { Node, mergeAttributes } from '@tiptap/core'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import FigureView from '@/components/FigureView.vue'

export const Figure = Node.create({
  name: 'figure',
  group: 'block',
  atom: true,
  draggable: false,
  isolating: true,

  addAttributes() {
    return {
      src: { default: '' },
      alt: { default: '' },
      caption: { default: '' },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'figure',
        getAttrs: (el) => {
          const img = el.querySelector('img')
          const cap = el.querySelector('figcaption')
          return {
            src: img?.getAttribute('src') ?? '',
            alt: img?.getAttribute('alt') ?? '',
            caption: cap?.textContent ?? '',
          }
        },
      },
    ]
  },

  renderHTML({ node }) {
    const cap = typeof node.attrs.caption === 'string' ? node.attrs.caption.trim() : ''
    const img = ['img', { src: node.attrs.src, alt: node.attrs.alt, loading: 'lazy' }]
    return cap.length
      ? ['figure', mergeAttributes({}), img, ['figcaption', cap]]
      : ['figure', mergeAttributes({}), img]
  },

  addNodeView() {
    return VueNodeViewRenderer(FigureView)
  },

  addCommands() {
    return {
      setFigure:
        (attrs) =>
        ({ chain }) =>
          chain().insertContent({ type: this.name, attrs }).run(),
    }
  },
})
