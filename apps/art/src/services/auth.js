export const mockUser = Object.freeze({
  id: 'visitor-10981',
  name: 'Observer',
  email: 'mock@haruhi.art',
  role: 'guest'
})

let currentUser = { ...mockUser }
const listeners = new Set()

function cloneUser(user) {
  return user ? { ...user } : null
}

function notify() {
  const nextUser = cloneUser(currentUser)
  listeners.forEach((listener) => listener(nextUser))
}

export const authService = {
  getUser() {
    return cloneUser(currentUser)
  },

  isLoggedIn() {
    return Boolean(currentUser)
  },

  login(email = mockUser.email) {
    currentUser = {
      ...mockUser,
      email: String(email || mockUser.email).trim() || mockUser.email
    }
    notify()
    return this.getUser()
  },

  logout() {
    currentUser = null
    notify()
  },

  subscribe(listener) {
    listeners.add(listener)
    return () => listeners.delete(listener)
  }
}
