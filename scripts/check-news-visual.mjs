import { mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'

const python = findPythonWithPlaywright()

if (!python) {
  console.error(
    'News visual checks require Python Playwright. Install it or set PYTHON to an interpreter that can import playwright.sync_api.'
  )
  process.exit(1)
}

const scriptPath = join(tmpdir(), 'haruhi-news-visual-check.py')

writeFileSync(
  scriptPath,
  String.raw`
from pathlib import Path
import os
import sys
from playwright.sync_api import sync_playwright

base_url = os.environ.get("NEWS_URL", "http://localhost:5204/news/")
out = Path(os.environ.get("NEWS_VISUAL_OUT", "tmp/visual-checks/news"))
out.mkdir(parents=True, exist_ok=True)

articles = [
    {
        "id": 1,
        "type": "news",
        "title": "北高校园祭筹备进入最终检查",
        "subtitle": "活动组完成摊位、排队和志愿者排班复核",
        "preview": "团报页面强调标题、摘要与日期。置顶文章要在不依赖 hover 的情况下显示专题、参与者、标签和发布时间。",
        "summary": "团报页面强调标题、摘要与日期。",
        "date": "2026-06-23",
        "isPinned": True,
        "pinOrder": 1,
        "tags": ["活动", "校园祭", "公告"],
        "participants": [
            {"name": "执行组", "role": "现场检查", "project": "校园祭"},
            {"name": "后勤组", "role": "物料确认", "project": "摊位"},
        ],
        "image": "/news/春日团报黑.webp",
    },
    {
        "id": 2,
        "type": "news",
        "title": "凉宫春日动画台词匹配站发布",
        "subtitle": "支持凉宫与京阿尼作品台词查询",
        "preview": "工具类项目需要把来源、发布时间和标签放在用户可以扫读的位置。界面不能用装饰盖过真实内容。",
        "date": "2026-06-20",
        "tags": ["技术", "工具", "发布"],
        "participants": [{"name": "开发组", "role": "检索系统", "project": "台词匹配"}],
        "image": "/news/春日团报白.webp",
    },
    {
        "id": 3,
        "type": "post",
        "title": "长门有希的书架维护记录",
        "subtitle": "阅读区目录、书签与返回动作调整",
        "preview": "长文和书目需要连续阅读线索。摘要区域保持高对比，标签和日期保留在卡片底部。",
        "date": "2026-06-18",
        "author": "长门有希的书架",
        "tags": ["阅读", "书架", "维护"],
    },
    {
        "id": 4,
        "type": "news",
        "title": "投稿审核规则更新：授权字段必须完整",
        "subtitle": "个人作品、网络转载和社团授权分开记录",
        "preview": "上传、审核和授权必须有明确文字证据。错误、空状态和待处理状态不能只用颜色表达。",
        "date": "2026-06-16",
        "tags": ["投稿", "审核", "授权"],
        "participants": [{"name": "美术部审核", "role": "授权核验", "project": "作品投稿"}],
        "image": "/news/春日小人.webp",
    },
    {
        "id": 5,
        "type": "post",
        "title": "奖品兑换库存每周复核",
        "subtitle": "预售、现货和发货状态必须常驻",
        "preview": "交易信息需要被比较和追踪。价格、库存和订单状态应当在商城切片中进一步重构。",
        "date": "2026-06-12",
        "author": "春日商城",
        "tags": ["商城", "库存", "订单"],
    },
]

api_payloads = {
    "/api/news/articles": {"message": "success", "data": articles},
    "/api/news/prizes": {"message": "success", "data": []},
    "/api/news/activities": {"message": "success", "data": []},
}

def fulfill(route):
    url = route.request.url
    for suffix, payload in api_payloads.items():
        if suffix in url:
            route.fulfill(status=200, content_type="application/json", json=payload)
            return
    route.continue_()

def check_page(page, name):
    page.goto(base_url, wait_until="networkidle")
    path = out / f"{name}.png"
    page.screenshot(path=str(path), full_page=True)
    metrics = page.evaluate("""() => ({
        overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        cardCount: document.querySelectorAll('.news-card').length,
        hasHero: !!document.querySelector('.news-hero'),
        firstCardText: document.querySelector('.news-card')?.innerText || '',
    })""")
    print(f"{name}: {metrics}")
    if metrics["overflowX"] != 0:
        raise AssertionError(f"{name} overflowX={metrics['overflowX']}")
    if metrics["cardCount"] < 4:
        raise AssertionError(f"{name} cardCount={metrics['cardCount']}")
    if not metrics["hasHero"]:
        raise AssertionError(f"{name} missing .news-hero")
    if "2026-06-23" not in metrics["firstCardText"] or "置顶" not in metrics["firstCardText"]:
        raise AssertionError(f"{name} first card missing stable date or pinned state")
    return path

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    try:
        for name, size in [
            ("desktop", {"width": 1280, "height": 1200}),
            ("mobile", {"width": 390, "height": 1100}),
        ]:
            page = browser.new_page(viewport=size)
            page.route("**/api/**", fulfill)
            path = check_page(page, name)
            print(f"screenshot: {path}")
            page.close()

        page = browser.new_page(viewport={"width": 1280, "height": 900})
        page.route("**/api/**", fulfill)
        page.goto(base_url, wait_until="networkidle")
        page.hover(".news-card")
        hover_path = out / "desktop-hover.png"
        page.screenshot(path=str(hover_path), full_page=False)
        print(f"screenshot: {hover_path}")
        page.close()
    finally:
        browser.close()
`
)

const result = spawnSync(python, [scriptPath], {
  stdio: 'inherit',
  shell: false,
  env: process.env,
})

process.exit(result.status ?? 1)

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
