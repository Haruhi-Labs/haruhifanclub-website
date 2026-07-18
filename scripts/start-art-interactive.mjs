import { spawn, spawnSync } from 'node:child_process'

const backendUrl = 'http://127.0.0.1:17777'
const healthUrl = `${backendUrl}/api/health`
const galleryUrl = 'http://127.0.0.1:5203/art/'
const canUseProcessGroups = process.platform !== 'win32'

let backend = null
let vite = null
let shuttingDown = false

const sleep = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds))

async function isBackendReady() {
  try {
    const response = await fetch(healthUrl)
    return response.ok
  } catch {
    return false
  }
}

async function isGalleryReady() {
  try {
    const response = await fetch(galleryUrl)
    if (!response.ok) return false
    const html = await response.text()
    return html.includes('data-sos-site="art"') && html.includes('<div id="app"></div>')
  } catch {
    return false
  }
}

function run(command, args, options = {}) {
  return spawn(command, args, {
    cwd: process.cwd(),
    detached: canUseProcessGroups,
    stdio: 'inherit',
    ...options,
  })
}

function stop(child, signal = 'SIGTERM') {
  if (!child?.pid || child.exitCode !== null || child.signalCode !== null) return

  try {
    if (canUseProcessGroups) process.kill(-child.pid, signal)
    else child.kill(signal)
  } catch {
    // 子进程可能刚好已经自行退出。
  }
}

function waitForExit(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return Promise.resolve()
  return new Promise(resolve => child.once('exit', resolve))
}

async function shutdown(code, signal = 'SIGTERM') {
  if (shuttingDown) return
  shuttingDown = true

  stop(vite, signal)
  stop(backend, signal)

  await Promise.race([
    Promise.all([waitForExit(vite), waitForExit(backend)]),
    sleep(1_500),
  ])
  process.exit(code)
}

async function waitForBackend(child) {
  const timeoutAt = Date.now() + 90_000

  while (Date.now() < timeoutAt) {
    if (await isBackendReady()) return
    if (child.exitCode !== null || child.signalCode !== null) {
      throw new Error(`本地后端提前退出（退出码 ${child.exitCode ?? child.signalCode}）`)
    }
    await sleep(500)
  }

  throw new Error('本地后端未能在 90 秒内启动')
}

async function main() {
  if ((await isBackendReady()) && (await isGalleryReady())) {
    process.stdout.write(`✓ 本地可交互画廊已在运行：${galleryUrl}\n`)
    return
  }

  if (await isBackendReady()) {
    process.stdout.write(`✓ 复用已运行的本地后端：${backendUrl}\n`)
  } else {
    process.stdout.write('正在启动本地后端…\n')
    backend = run('pnpm', ['dev:backend'])
    await new Promise((resolve, reject) => {
      backend.once('spawn', resolve)
      backend.once('error', reject)
    })
    await waitForBackend(backend)
  }

  const sync = spawnSync(
    process.execPath,
    ['--disable-warning=ExperimentalWarning', 'scripts/sync-art-public-snapshot.mjs'],
    {
      cwd: process.cwd(),
      env: process.env,
      stdio: 'inherit',
    },
  )
  if (sync.error) throw sync.error
  if (sync.status !== 0) throw new Error(`真实作品同步失败（退出码 ${sync.status ?? 1}）`)

  vite = run(
    'pnpm',
    ['--filter', '@haruhi/art', 'exec', 'vite', '--port', '5203', '--strictPort'],
    {
      env: {
        ...process.env,
        VITE_ART_PROXY_TARGET: backendUrl,
        VITE_ART_UPLOADS_PROXY_TARGET: 'https://haruyuki.cn',
        VITE_ART_THUMB_PROXY_TARGET: 'https://haruyuki.cn',
      },
    },
  )

  vite.once('error', error => {
    process.stderr.write(`✗ 本地画廊启动失败：${error.message}\n`)
    void shutdown(1)
  })
  vite.once('exit', (code, signal) => {
    if (shuttingDown) return
    void shutdown(code ?? (signal ? 1 : 0))
  })

  if (backend) {
    backend.once('exit', (code, signal) => {
      if (shuttingDown) return
      process.stderr.write(`✗ 本地后端意外退出（退出码 ${code ?? signal}）\n`)
      void shutdown(1)
    })
  }
}

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.once(signal, () => void shutdown(0, signal))
}

try {
  await main()
} catch (error) {
  process.stderr.write(`✗ ${error?.message || error}\n`)
  await shutdown(1)
}
