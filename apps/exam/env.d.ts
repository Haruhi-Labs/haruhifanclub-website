/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'html-to-image' {
  export function toPng(node: HTMLElement, options?: any): Promise<string>
  export function toJpeg(node: HTMLElement, options?: any): Promise<string>
  export function toBlob(node: HTMLElement, options?: any): Promise<Blob | null>
  export function toSvg(node: HTMLElement, options?: any): Promise<string>
}
