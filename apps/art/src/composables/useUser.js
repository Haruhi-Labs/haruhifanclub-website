import { computed, readonly, shallowRef } from 'vue'
import { authService } from '../services/auth.js'

const userState = shallowRef(authService.getUser())

authService.subscribe((nextUser) => {
  userState.value = nextUser
})

export function useUser() {
  const user = readonly(userState)

  return {
    user,
    isLoggedIn: computed(() => Boolean(user.value)),
    userId: computed(() => user.value?.id || ''),
    displayName: computed(() => user.value?.name || 'Guest'),
    email: computed(() => user.value?.email || ''),
    role: computed(() => user.value?.role || 'guest'),
    login: authService.login.bind(authService),
    logout: authService.logout.bind(authService)
  }
}
