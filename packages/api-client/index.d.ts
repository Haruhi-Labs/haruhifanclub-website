export interface RequestOptions {
  body?: unknown
  headers?: Record<string, string>
  isForm?: boolean
  signal?: AbortSignal
}

export interface ApiError extends Error {
  status: number
  data: unknown
}

export interface ApiClient {
  base: string
  get<T = any>(path: string, opts?: RequestOptions): Promise<T>
  post<T = any>(path: string, body?: unknown, opts?: RequestOptions): Promise<T>
  put<T = any>(path: string, body?: unknown, opts?: RequestOptions): Promise<T>
  patch<T = any>(path: string, body?: unknown, opts?: RequestOptions): Promise<T>
  del<T = any>(path: string, opts?: RequestOptions): Promise<T>
  postForm<T = any>(path: string, formData: FormData, opts?: RequestOptions): Promise<T>
}

export interface AppRole {
  role: string
  roleName: string
  level: number
}

export interface CurrentUser {
  id: number
  username: string
  displayName: string | null
  isSuperAdmin: boolean
  apps: Record<string, AppRole>
}

export interface Auth {
  login(username: string, password: string): Promise<CurrentUser>
  me(): Promise<CurrentUser>
  logout(): void
  getToken(): string
  isLoggedIn(): boolean
}

export interface LoginResult {
  ok: boolean
  user?: CurrentUser
  error?: string
}

export interface AdminAuth {
  app: string
  hasPerm(user: CurrentUser | null | undefined): boolean
  hasValidToken(): boolean
  buildHeaders(headers?: Record<string, string>): Record<string, string>
  login(username: string, password: string): Promise<LoginResult>
  restore(): Promise<CurrentUser | null>
  me(): Promise<CurrentUser>
  logout(): void
  getToken(): string
  isLoggedIn(): boolean
}

export function getToken(): string
export function setToken(token: string): void
export function clearToken(): void
export function createApiClient(base?: string): ApiClient
export function createAuth(apiBase?: string): Auth
export function createAdminAuth(app: string, apiBase?: string): AdminAuth
export function hasScope(user: CurrentUser | null | undefined, scope: string): boolean
