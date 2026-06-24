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
  login(account: string, password: string): Promise<SessionUser>
  register(payload: { email: string; password: string; nickname?: string }): Promise<SessionUser>
  logout(): Promise<void>
  updateProfile(patch: { nickname?: string; avatar?: string; bio?: string }): Promise<SessionUser>
  forgotPassword(email: string): Promise<unknown>
  resetPassword(token: string, password: string): Promise<unknown>
  verifyEmail(token: string): Promise<unknown>
  resendVerification(): Promise<unknown>
  changePassword(oldPassword: string, newPassword: string): Promise<unknown>
  listSessions(): Promise<{ sessions: any[] }>
  revokeSession(id: string): Promise<unknown>
}

export function useSession(apiBase?: string): SessionApi

export const LoginView: DefineComponent<Record<string, any>>
export const ResetPasswordView: DefineComponent<Record<string, any>>
export const VerifyEmailView: DefineComponent<Record<string, any>>
export const ProfileView: DefineComponent<Record<string, any>>
export const SettingsView: DefineComponent<Record<string, any>>
export const AccountMenu: DefineComponent<Record<string, any>>
