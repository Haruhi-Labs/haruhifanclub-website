import { spawnSync } from 'node:child_process'

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm'
const args = new Set(process.argv.slice(2))
const withBrowser = args.has('--browser')

const checks = [
  [
    'prettier',
    [
      'exec',
      'prettier',
      '--check',
      'package.json',
      'apps/design-system/src/main.js',
      'apps/design-system/src/style.css',
      'docs/DESIGN_SYSTEM.md',
      'packages/design-system/README.md',
      'packages/ui/README.md',
      'scripts/check-design-system.mjs',
    ],
  ],
  ['eslint', ['exec', 'eslint', 'apps/design-system/src/main.js', '--max-warnings=0']],
  ['design-system build', ['--filter', '@haruhi/design-system-docs', 'build']],
  ['ui typecheck', ['--filter', '@haruhi/ui', 'typecheck']],
]

function run(label, command, commandArgs, options = {}) {
  console.log(`\n==> ${label}`)
  const result = spawnSync(command, commandArgs, {
    stdio: 'inherit',
    shell: false,
    ...options,
  })

  if (result.status !== 0) {
    process.exit(result.status ?? 1)
  }
}

for (const [label, commandArgs] of checks) {
  run(label, pnpm, commandArgs)
}

if (!withBrowser) {
  console.log('\nDesign system checks passed.')
  console.log(
    'Run `pnpm check:design-system:browser` with the docs dev server running for viewport checks.'
  )
  process.exit(0)
}

const python = findPythonWithPlaywright()
if (!python) {
  console.error(
    '\nBrowser checks require Python Playwright. Install it or set PYTHON to an interpreter that can import playwright.sync_api.'
  )
  process.exit(1)
}

const browserCheck = String.raw`
import os
import sys
from playwright.sync_api import sync_playwright

base_url = os.environ.get("DESIGN_SYSTEM_URL", "http://127.0.0.1:5206/design-system/")
sections = os.environ.get(
    "DESIGN_SYSTEM_SECTIONS",
    "overview,color,type,space,buttons,forms,data,feedback,overlay,nav,expressions,patterns",
).split(",")
widths = [int(value) for value in os.environ.get("DESIGN_SYSTEM_WIDTHS", "390,768,1280,1440").split(",")]

failures = []

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page()
    for section in sections:
        target = f"{base_url.rstrip('/')}/#{section}"
        for width in widths:
            page.set_viewport_size({"width": width, "height": 1500})
            page.goto(target, wait_until="networkidle")
            result = page.evaluate("""() => ({
              overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
              bodyOverflowX: document.body.scrollWidth - document.body.clientWidth
            })""")
            if result["overflowX"] != 0 or result["bodyOverflowX"] != 0:
                failures.append((section, width, result))
            print(f"{section} {width}px overflowX={result['overflowX']} bodyOverflowX={result['bodyOverflowX']}")
    browser.close()

if failures:
    print("\nViewport overflow failures:", file=sys.stderr)
    for section, width, result in failures:
        print(f"- {section} at {width}px: {result}", file=sys.stderr)
    sys.exit(1)
`

run('browser viewport checks', python, ['-c', browserCheck], {
  env: {
    ...process.env,
  },
})

console.log('\nDesign system checks passed.')

function findPythonWithPlaywright() {
  const candidates = [
    process.env.PYTHON,
    '/opt/anaconda3/bin/python',
    process.platform === 'win32' ? 'python.exe' : 'python3',
    process.platform === 'win32' ? 'py.exe' : 'python',
  ].filter(Boolean)

  for (const candidate of candidates) {
    const result = spawnSync(candidate, ['-c', 'import playwright.sync_api'], {
      stdio: 'ignore',
      shell: false,
    })

    if (result.status === 0) {
      return candidate
    }
  }

  return null
}
