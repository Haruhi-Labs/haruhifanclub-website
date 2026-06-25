import type { DefineComponent, ComputedRef } from 'vue'

/** 当前登录用户档案（与后端 /api/auth/me 一致） */
export interface SessionUser {
  id: number
  username: string
  displayName: string | null
  nickname: string | null
  avatar: string | null
  bio: string | null
  email: string | null
  emailVerified: boolean
  isSuperAdmin: boolean
  apps: Record<string, { role: string; roleName: string; level: number }>
}

export interface SessionApi {
  state: { user: SessionUser | null; ready: boolean; loading: boolean }
  isLoggedIn: ComputedRef<boolean>
  isVerified: ComputedRef<boolean>
  isSuperAdmin: ComputedRef<boolean>
  refresh(): Promise<SessionUser | null>
  login(_account: string, _password: string): Promise<SessionUser>
  register(_payload: { email: string; password: string; nickname?: string }): Promise<SessionUser>
  logout(): Promise<void>
  updateProfile(_patch: { nickname?: string; avatar?: string; bio?: string }): Promise<SessionUser>
  forgotPassword(_email: string): Promise<unknown>
  resetPassword(_token: string, _password: string): Promise<unknown>
  verifyEmail(_token: string): Promise<unknown>
  resendVerification(): Promise<unknown>
  changePassword(_oldPassword: string, _newPassword: string): Promise<unknown>
  listSessions(): Promise<{ sessions: any[] }>
  revokeSession(_id: string): Promise<unknown>
}

export function useSession(_apiBase?: string): SessionApi

/** 个人中心数据访问层：统一封装各模块 /me/* 与跨库 /me/summary。 */
export interface UserHub {
  summary(): Promise<any>
  art: Record<string, (..._args: any[]) => Promise<any>>
  news: Record<string, (..._args: any[]) => Promise<any>>
  exam: Record<string, (..._args: any[]) => Promise<any>>
}
export function useUserHub(_apiBase?: string): UserHub

/** 个人控制台上下文（UserConsoleLayout 经 provide 下发）。 */
export interface ConsoleContext {
  apiBase: string
  site?: string
  basePath: string
  loginPath: string
  home: string
}
export function useConsoleContext(): ConsoleContext

export const LoginView: DefineComponent<Record<string, any>>
export const ResetPasswordView: DefineComponent<Record<string, any>>
export const VerifyEmailView: DefineComponent<Record<string, any>>
export const ProfileView: DefineComponent<Record<string, any>>
export const SettingsView: DefineComponent<Record<string, any>>
export const AccountMenu: DefineComponent<Record<string, any>>
export const UserConsoleLayout: DefineComponent<Record<string, any>>
export const OverviewView: DefineComponent<Record<string, any>>
export const MyArtworksView: DefineComponent<Record<string, any>>
