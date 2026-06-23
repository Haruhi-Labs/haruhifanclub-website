import type { DefineComponent } from 'vue'

export type SosButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger'
export type SosButtonSize = 'sm' | 'md' | 'lg'
export type SosBadgeVariant = 'default' | 'accent' | 'solid' | 'outline' | 'signal'
export type SosNoticeTone = 'info' | 'success' | 'warning' | 'danger'
export type SosStackGap = 'default' | 'tight' | 'loose'
export type SosMediaRatio = '1:1' | '4:3' | '3:4' | '2:3'
export type SosSurfaceVariant = 'default' | 'subtle' | 'raised'
export type SosProgressTone = 'default' | 'success' | 'danger'
export type SosClusterAlign = 'start' | 'center' | 'end' | 'stretch'
export type SosClusterJustify = 'start' | 'center' | 'end' | 'between'
export type SosSplitAlign = 'start' | 'center' | 'end' | 'stretch'

export const SosBadge: DefineComponent<{
  variant?: SosBadgeVariant
  selected?: boolean
  disabled?: boolean
}>
export const SosButton: DefineComponent<{
  as?: 'button' | 'a'
  variant?: SosButtonVariant
  size?: SosButtonSize
  type?: 'button' | 'submit' | 'reset'
  href?: string
  disabled?: boolean
  loading?: boolean
}>
export const SosCard: DefineComponent<{
  as?: string
  interactive?: boolean
  selected?: boolean
  loading?: boolean
}>
export const SosCluster: DefineComponent<{
  as?: string
  align?: SosClusterAlign
  justify?: SosClusterJustify
  gap?: string
}>
export const SosEmptyState: DefineComponent<{
  title: string
  copy?: string
}>
export const SosField: DefineComponent<{
  label?: string
  help?: string
  error?: string
  required?: boolean
  forId?: string
}>
export const SosHeaderBrand: DefineComponent<{
  as?: string
  href?: string
  logoSrc?: string
  logoAlt?: string
  title: string
  subtitle?: string
  compact?: boolean
}>
export const SosGrid: DefineComponent<{
  as?: string
  min?: string
  gap?: string
}>
export const SosInline: DefineComponent<{
  as?: string
  gap?: SosStackGap
}>
export const SosMediaFrame: DefineComponent<{
  as?: string
  ratio?: SosMediaRatio
}>
export const SosNotice: DefineComponent<{
  title?: string
  tone?: SosNoticeTone
}>
export const SosProgress: DefineComponent<{
  value: number
  max?: number
  label?: string
  valueLabel?: string
  tone?: SosProgressTone
}>
export const SosSplit: DefineComponent<{
  as?: string
  left?: string
  right?: string
  gap?: string
  align?: SosSplitAlign
}>
export const SosStack: DefineComponent<{
  as?: string
  gap?: SosStackGap
}>
export const SosSurface: DefineComponent<{
  as?: string
  variant?: SosSurfaceVariant
  padded?: boolean
}>
