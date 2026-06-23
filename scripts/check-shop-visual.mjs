import { mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'

const python = findPythonWithPlaywright()

if (!python) {
  console.error(
    'Shop visual checks require Python Playwright. Install it or set PYTHON to an interpreter that can import playwright.sync_api.'
  )
  process.exit(1)
}

const scriptPath = join(tmpdir(), 'haruhi-shop-visual-check.py')

writeFileSync(
  scriptPath,
  String.raw`
from pathlib import Path
from urllib.parse import quote
import os
from playwright.sync_api import sync_playwright

base_url = os.environ.get("SHOP_URL", "http://localhost:5205/shop/")
out = Path(os.environ.get("SHOP_VISUAL_OUT", "tmp/visual-checks/shop"))
out.mkdir(parents=True, exist_ok=True)

def svg_data(bg, accent, label):
    svg = f"""
    <svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 900 900'>
      <rect width='900' height='900' fill='{bg}'/>
      <circle cx='680' cy='180' r='86' fill='{accent}' opacity='.34'/>
      <rect x='170' y='150' width='560' height='560' rx='80' fill='#fff' stroke='#dde1e6' stroke-width='12'/>
      <rect x='250' y='270' width='400' height='260' rx='48' fill='{accent}' opacity='.82'/>
      <path d='M302 610h296' stroke='#171a22' stroke-width='24' stroke-linecap='round'/>
      <text x='450' y='452' text-anchor='middle' font-family='Arial, sans-serif' font-size='56' font-weight='700' fill='#171a22'>{label}</text>
    </svg>
    """
    return "data:image/svg+xml;charset=UTF-8," + quote(svg)

products = [
    {
        "id": 1,
        "name": "朝比奈实玖瑠 fufu",
        "desc": "达标后统一排产，订单持续累计中。",
        "price": 168,
        "discountPrice": 147,
        "stock": 0,
        "category": "fufu",
        "image": svg_data("#eef4ff", "#ffc83d", "fufu"),
        "imageMobile": svg_data("#eef4ff", "#ffc83d", "fufu"),
        "shippingCost": 12,
        "presaleMode": "goal",
        "presaleGoalTarget": 200,
        "presalePaidCount": 126,
        "specs": [
            {"key": "尺寸", "val": "约 20cm"},
            {"key": "发货", "val": "达标后统一排产"}
        ],
        "detailText": "实玖瑠 fufu 预售中，达标后统一排产。",
        "detailImages": [svg_data("#f7f8f9", "#3478f6", "detail")]
    },
    {
        "id": 2,
        "name": "SOS 团限定徽章",
        "desc": "现货少量补充，适合作为活动签到纪念。",
        "price": 42,
        "discountPrice": None,
        "stock": 18,
        "category": "徽章",
        "image": svg_data("#fff9db", "#3478f6", "badge"),
        "imageMobile": svg_data("#fff9db", "#3478f6", "badge"),
        "shippingCost": 8,
        "presaleMode": "none",
        "specs": [{"key": "材质", "val": "金属烤漆"}],
        "detailText": "现货少量补充，适合作为活动签到纪念。",
        "detailImages": []
    },
    {
        "id": 3,
        "name": "北高校园祭贴纸组",
        "desc": "排期预售，适合活动前集中发货。",
        "price": 28,
        "discountPrice": 24,
        "stock": 0,
        "category": "贴纸",
        "image": svg_data("#fff4e5", "#61c8a9", "sticker"),
        "imageMobile": svg_data("#fff4e5", "#61c8a9", "sticker"),
        "shippingCost": 6,
        "presaleMode": "fixed",
        "presaleFixedDateType": "month_end",
        "presaleFixedDateValue": "2026-07",
        "specs": [{"key": "规格", "val": "12 枚/组"}],
        "detailText": "排期预售，活动前集中发货。",
        "detailImages": []
    },
    {
        "id": 4,
        "name": "团长推荐文件夹",
        "desc": "日常文具周边，适合与徽章一起结算。",
        "price": 36,
        "discountPrice": None,
        "stock": 32,
        "category": "文具",
        "image": svg_data("#faf6ec", "#db3a46", "file"),
        "imageMobile": svg_data("#faf6ec", "#db3a46", "file"),
        "shippingCost": 10,
        "presaleMode": "none",
        "specs": [{"key": "尺寸", "val": "A4"}],
        "detailText": "日常文具周边，适合与徽章一起结算。",
        "detailImages": []
    },
]

api_payloads = {
    "/api/shop/products": products,
    "/api/shop/site-config": {"payment": {"wechatQr": "", "alipayQr": "", "friendQr": ""}},
}

def fulfill(route):
    url = route.request.url
    for suffix, payload in api_payloads.items():
        if suffix in url:
            route.fulfill(status=200, content_type="application/json", json=payload)
            return
    route.fulfill(status=200, content_type="application/json", json={"ok": True})

def visible_text(page, selector):
    return page.locator(selector).first.inner_text(timeout=5000)

def check_home(page, name):
    page.goto(base_url, wait_until="networkidle")
    path = out / f"{name}.png"
    page.screenshot(path=str(path), full_page=True)
    metrics = page.evaluate("""() => {
        const first = document.querySelector('.shop-product-card')
        const overlay = document.querySelector('.card-overlay')
        return {
            overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
            cardCount: document.querySelectorAll('.shop-product-card').length,
            firstCardText: first?.innerText || '',
            visibleActions: Array.from(document.querySelectorAll('.shop-card-action')).filter((el) => {
                const rect = el.getBoundingClientRect()
                return rect.width > 0 && rect.height > 0
            }).length,
            overlayDisplay: overlay ? getComputedStyle(overlay).display : 'missing',
            hasHeaderBrand: !!document.querySelector('.shop-header-brand .brand-logo-img'),
        }
    }""")
    print(f"{name}: {metrics}")
    if metrics["overflowX"] != 0:
        raise AssertionError(f"{name} overflowX={metrics['overflowX']}")
    if metrics["cardCount"] < 4:
        raise AssertionError(f"{name} cardCount={metrics['cardCount']}")
    first_text = metrics["firstCardText"]
    for expected in ["朝比奈实玖瑠 fufu", "¥147", "¥168", "进度预售", "126/200", "预售商品", "加入购物车"]:
        if expected not in first_text:
            raise AssertionError(f"{name} first card missing {expected}")
    if metrics["visibleActions"] < 4:
        raise AssertionError(f"{name} add-to-cart actions are not persistently visible")
    if metrics["overlayDisplay"] != "missing":
        raise AssertionError(f"{name} still has hover overlay display={metrics['overlayDisplay']}")
    if not metrics["hasHeaderBrand"]:
        raise AssertionError(f"{name} missing header logo lockup")
    return path

def check_detail(page):
    page.goto(base_url + "product/1", wait_until="networkidle")
    path = out / "detail.png"
    page.screenshot(path=str(path), full_page=True)
    metrics = page.evaluate("""() => ({
        overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        title: document.querySelector('.detail-title')?.innerText || '',
        price: document.querySelector('.price-box')?.innerText || '',
        presale: document.querySelector('.detail-presale-card')?.innerText || '',
        actions: Array.from(document.querySelectorAll('.action-row .sos-button')).map((el) => el.innerText).join(' / '),
        hasHeaderBrand: !!document.querySelector('.mini-header-detail .shop-header-brand .brand-logo-img'),
    })""")
    print(f"detail: {metrics}")
    if metrics["overflowX"] != 0:
        raise AssertionError(f"detail overflowX={metrics['overflowX']}")
    if "朝比奈实玖瑠 fufu" not in metrics["title"]:
        raise AssertionError("detail missing product title")
    for expected in ["应援价", "¥147", "¥168", "预售"]:
        if expected not in metrics["price"]:
            raise AssertionError(f"detail price box missing {expected}")
    for expected in ["开做条件", "已支付 126 / 200"]:
        if expected not in metrics["presale"]:
            raise AssertionError(f"detail presale missing {expected}")
    for expected in ["立即购买", "加入购物车"]:
        if expected not in metrics["actions"]:
            raise AssertionError(f"detail actions missing {expected}")
    if not metrics["hasHeaderBrand"]:
        raise AssertionError("detail missing header logo lockup")
    return path

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    try:
        for name, size in [
            ("home-desktop", {"width": 1280, "height": 1100}),
            ("home-mobile", {"width": 390, "height": 1200}),
        ]:
            page = browser.new_page(viewport=size)
            page.route("**/api/**", fulfill)
            path = check_home(page, name)
            print(f"screenshot: {path}")
            page.close()

        page = browser.new_page(viewport={"width": 1280, "height": 1100})
        page.route("**/api/**", fulfill)
        path = check_detail(page)
        print(f"screenshot: {path}")
        page.close()

        page = browser.new_page(viewport={"width": 1280, "height": 900})
        page.route("**/api/**", fulfill)
        page.goto(base_url, wait_until="networkidle")
        page.hover(".shop-product-card")
        hover_path = out / "home-hover.png"
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
