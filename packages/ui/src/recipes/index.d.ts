import type { DefineComponent } from 'vue'

export interface SosProductProgress {
  value: number
  max: number
  label?: string
  valueLabel?: string
}

export const SosArticleCard: DefineComponent<{
  label?: string
  title: string
  subtitle?: string
  excerpt?: string
  image?: string
  imageAlt?: string
  date?: string
  author?: string
  tags?: string[]
  pinned?: boolean
  interactive?: boolean
}>
export const SosProductCard: DefineComponent<{
  title: string
  desc?: string
  image?: string
  imageAlt?: string
  price: number | string
  originalPrice?: number | string
  badge?: string
  badgeTone?: 'accent' | 'signal' | 'danger'
  soldOut?: boolean
  soldOutLabel?: string
  state?: string
  progress?: SosProductProgress
  interactive?: boolean
}>
export const SosArtworkCard: DefineComponent<{
  title: string
  author?: string
  image?: string
  imageAlt?: string
  ratio?: '4:3' | '1:1' | '3:4' | '16:9'
  likes?: number
  views?: number
}>
export const SosBookCard: DefineComponent<{
  title: string
  author?: string
  cover?: string
  color?: string
  badge?: string
}>
export const SosExamCard: DefineComponent<{
  title: string
  subject?: string
  score?: string
  meta?: string[]
  interactive?: boolean
}>
