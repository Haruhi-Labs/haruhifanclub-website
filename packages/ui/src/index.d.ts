import type { DefineComponent, Ref } from 'vue'

export type SosButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'link'
export type SosButtonSize = 'sm' | 'md' | 'lg'
export type SosBadgeVariant =
  | 'default'
  | 'accent'
  | 'solid'
  | 'outline'
  | 'signal'
  | 'success'
  | 'danger'
  | 'info'
export type SosNoticeTone = 'info' | 'success' | 'warning' | 'danger'
export type SosStackGap = 'default' | 'tight' | 'loose'
export type SosMediaRatio = '1:1' | '4:3' | '16:9' | '3:4' | '2:3'
export type SosSurfaceVariant = 'default' | 'subtle' | 'inset' | 'raised' | 'floating'
export type SosProgressTone = 'default' | 'success' | 'danger'
export type SosClusterAlign = 'start' | 'center' | 'end' | 'stretch'
export type SosClusterJustify = 'start' | 'center' | 'end' | 'between'
export type SosSplitAlign = 'start' | 'center' | 'end' | 'stretch'
export type SosPageContained = 'none' | 'content' | 'wide' | 'reading'
export type SosPageGap = 'default' | 'tight' | 'loose'
export type SosTitleSize = 'default' | 'xl' | 'hero'
export type SosCopyVariant = 'default' | 'lead' | 'small'
export type SosTabsVariant = 'pill' | 'underline'
export type SosAvatarSize = 'sm' | 'md' | 'lg'
export type SosSpinnerSize = 'md' | 'lg'
export type SosSkeletonVariant = 'text' | 'title' | 'block' | 'circle'

export interface SosSelectOption {
  label: string
  value: string | number
  disabled?: boolean
}
export interface SosTabItem {
  value: string | number
  label: string
  disabled?: boolean
}
export interface SosCrumb {
  label: string
  href?: string
}
export interface SosColumn {
  key: string
  label: string
  numeric?: boolean
}

/* Layout & page structure */
export const SosPage: DefineComponent<{
  as?: string
  site?: string
  density?: string
  contained?: SosPageContained
  gap?: SosPageGap
  scoped?: boolean
}>
export const SosPageHeader: DefineComponent<{
  as?: string
  eyebrow?: string
  title?: string
  copy?: string
  meta?: string
}>
export const SosToolbar: DefineComponent<{ as?: string; surface?: boolean }>
export const SosStack: DefineComponent<{ as?: string; gap?: SosStackGap }>
export const SosInline: DefineComponent<{ as?: string; gap?: SosStackGap }>
export const SosCluster: DefineComponent<{
  as?: string
  align?: SosClusterAlign
  justify?: SosClusterJustify
  gap?: string
}>
export const SosGrid: DefineComponent<{ as?: string; min?: string; gap?: string }>
export const SosSplit: DefineComponent<{
  as?: string
  left?: string
  right?: string
  gap?: string
  align?: SosSplitAlign
}>
export const SosSurface: DefineComponent<{
  as?: string
  variant?: SosSurfaceVariant
  padded?: boolean
}>
export const SosMediaFrame: DefineComponent<{ as?: string; ratio?: SosMediaRatio }>
export const SosDivider: DefineComponent<{ label?: string }>

/* Typography helpers */
export const SosEyebrow: DefineComponent<{ as?: string; plain?: boolean }>
export const SosTitle: DefineComponent<{ as?: string; size?: SosTitleSize }>
export const SosCopy: DefineComponent<{ as?: string; variant?: SosCopyVariant }>

/* Controls */
export const SosButton: DefineComponent<{
  as?: 'button' | 'a'
  variant?: SosButtonVariant
  size?: SosButtonSize
  type?: 'button' | 'submit' | 'reset'
  href?: string
  disabled?: boolean
  loading?: boolean
}>
export const SosBadge: DefineComponent<{
  variant?: SosBadgeVariant
  selected?: boolean
  disabled?: boolean
}>
export const SosChip: DefineComponent<{
  pressed?: boolean
  removable?: boolean
  disabled?: boolean
}>
export const SosTabs: DefineComponent<{
  modelValue?: string | number
  items?: SosTabItem[]
  variant?: SosTabsVariant
}>

/* Forms */
export const SosField: DefineComponent<{
  label?: string
  help?: string
  error?: string
  required?: boolean
  forId?: string
}>
export const SosInput: DefineComponent<{
  modelValue?: string | number
  type?: string
  placeholder?: string
  invalid?: boolean
  disabled?: boolean
  id?: string
}>
export const SosTextarea: DefineComponent<{
  modelValue?: string
  placeholder?: string
  rows?: number
  invalid?: boolean
  disabled?: boolean
  id?: string
}>
export const SosSelect: DefineComponent<{
  modelValue?: string | number
  options?: SosSelectOption[]
  disabled?: boolean
  invalid?: boolean
  id?: string
}>
export const SosCheckbox: DefineComponent<{
  modelValue?: boolean | string | number
  type?: 'checkbox' | 'radio'
  value?: string | number
  name?: string
  disabled?: boolean
}>
export const SosSwitch: DefineComponent<{ modelValue?: boolean; disabled?: boolean }>

/* Data & media display */
export const SosCard: DefineComponent<{
  as?: string
  interactive?: boolean
  selected?: boolean
  loading?: boolean
}>
export const SosAvatar: DefineComponent<{
  src?: string
  alt?: string
  name?: string
  size?: SosAvatarSize
  square?: boolean
}>
export const SosTable: DefineComponent<{
  columns?: SosColumn[]
  rows?: Record<string, unknown>[]
  zebra?: boolean
  rowKey?: string
}>
export const SosTooltip: DefineComponent<{ label: string }>
export const SosSkeleton: DefineComponent<{
  variant?: SosSkeletonVariant
  width?: string
  height?: string
}>
export const SosSpinner: DefineComponent<{ size?: SosSpinnerSize; label?: string }>

/* Feedback */
export const SosNotice: DefineComponent<{ title?: string; tone?: SosNoticeTone }>
export const SosProgress: DefineComponent<{
  value: number
  max?: number
  label?: string
  valueLabel?: string
  tone?: SosProgressTone
}>
export const SosEmptyState: DefineComponent<{ title: string; copy?: string }>
export const SosToastRegion: DefineComponent<Record<string, never>>

/* Navigation */
export const SosAppbar: DefineComponent<{ as?: string }>
export const SosNavLink: DefineComponent<{ as?: string; href?: string; active?: boolean }>
export const SosBreadcrumb: DefineComponent<{ items?: SosCrumb[]; separator?: string }>
export const SosPagination: DefineComponent<{
  modelValue?: number
  pageCount: number
  siblings?: number
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

/* Overlay */
export const SosModal: DefineComponent<{
  open?: boolean
  title?: string
  wide?: boolean
  closeOnBackdrop?: boolean
}>
export const SosDropdown: DefineComponent<{ align?: 'start' | 'end' }>

/* Composables */
export type SosToastTone = 'default' | 'success' | 'danger'
export interface SosToast {
  id: number
  title?: string
  message: string
  tone: SosToastTone
}
export interface SosToastOptions {
  title?: string
  tone?: SosToastTone
  duration?: number
}
export interface SosToastApi {
  toasts: Ref<SosToast[]>
  push: (_message: string, _options?: SosToastOptions) => number
  dismiss: (_id: number) => void
  success: (_message: string, _options?: SosToastOptions) => number
  danger: (_message: string, _options?: SosToastOptions) => number
}
export function useToast(): SosToastApi
