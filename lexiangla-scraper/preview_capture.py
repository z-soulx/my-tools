"""Capture full-resolution preview images of document slides.

Strategy for maximum quality:
  1. Load the document page to discover the WebOffice iframe URL
  2. Navigate directly to the iframe URL in a maximized viewport
     (slide fills the full viewport instead of a small embed)
  3. Also intercept embedded shape images from previewsh.myqcloud.com
  4. Screenshot each slide at high resolution (3840x2160 viewport, 2x DPR)
"""

from __future__ import annotations

import re
from pathlib import Path

from playwright.sync_api import sync_playwright, Page, Frame, Response


def _build_cookies(cookie_str: str, domain: str = "lexiangla.com") -> list[dict]:
    cookies = []
    for pair in cookie_str.split(";"):
        pair = pair.strip()
        if "=" not in pair:
            continue
        name, value = pair.split("=", 1)
        cookies.append({
            "name": name.strip(),
            "value": value.strip(),
            "domain": f".{domain}",
            "path": "/",
        })
    return cookies


def capture_preview_images(
    doc_url: str,
    cookie_str: str,
    output_dir: Path,
    *,
    title: str = "preview",
    total_pages: int | None = None,
    headless: bool = True,
    timeout_ms: int = 60000,
) -> list[Path]:
    """Navigate through slides and capture at maximum resolution."""
    output_dir.mkdir(parents=True, exist_ok=True)
    safe_title = _safe(title)
    num_pages = total_pages or 1

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=headless, channel="chrome")

        ctx_parent = browser.new_context(
            viewport={"width": 1920, "height": 1080},
        )
        ctx_parent.add_cookies(_build_cookies(cookie_str))
        page_parent = ctx_parent.new_page()

        print("  [浏览器] 加载页面获取 iframe 地址...")
        page_parent.goto(doc_url, wait_until="networkidle", timeout=timeout_ms)
        page_parent.wait_for_timeout(6000)

        iframe_url = _find_iframe_url(page_parent)
        ctx_parent.close()

        if not iframe_url:
            print("  [浏览器] 未找到 WebOffice iframe，直接截取页面")
            ctx_fallback = browser.new_context(
                viewport={"width": 1920, "height": 1080},
                device_scale_factor=2,
            )
            ctx_fallback.add_cookies(_build_cookies(cookie_str))
            pg = ctx_fallback.new_page()
            pg.goto(doc_url, wait_until="networkidle", timeout=timeout_ms)
            pg.wait_for_timeout(5000)
            dest = output_dir / f"{safe_title}_full.png"
            pg.screenshot(path=str(dest), full_page=True)
            ctx_fallback.close()
            browser.close()
            return [dest]

        print(f"  [浏览器] iframe 地址已获取")

        ctx_hd = browser.new_context(
            viewport={"width": 3840, "height": 2160},
            device_scale_factor=2,
        )
        iframe_domain = _get_domain(iframe_url)
        if iframe_domain:
            ctx_hd.add_cookies(_build_cookies(cookie_str))
            ctx_hd.add_cookies([
                {"name": "weboffice_cdn", "value": "1", "domain": f".{iframe_domain}", "path": "/"},
                {"name": "lang", "value": "zh-CN", "domain": f".{iframe_domain}", "path": "/"},
            ])

        page_hd = ctx_hd.new_page()

        shape_images: dict[str, bytes] = {}

        def on_response(resp: Response) -> None:
            url = resp.url
            if "previewsh.myqcloud.com/shapes" not in url:
                return
            ct = resp.headers.get("content-type", "")
            if "image/" not in ct:
                return
            h = _extract_shape_hash(url)
            if h and h not in shape_images:
                try:
                    shape_images[h] = resp.body()
                except Exception:
                    pass

        page_hd.on("response", on_response)

        print(f"  [浏览器] 用 3840x2160@2x 加载预览 (输出 7680x4320)...")
        page_hd.goto(iframe_url, wait_until="networkidle", timeout=timeout_ms)
        page_hd.wait_for_timeout(5000)

        slide_el = _find_slide_element_on_page(page_hd)
        if slide_el:
            box = slide_el.bounding_box()
            if box:
                print(f"  [浏览器] 幻灯片区域: {int(box['width'])}x{int(box['height'])} CSS px")

        try:
            page_hd.click(".play_root_container", timeout=3000, force=True)
        except Exception:
            page_hd.click("body", force=True)
        page_hd.wait_for_timeout(500)

        saved: list[Path] = []
        print(f"  [浏览器] 开始截取 {num_pages} 页 (高清)...")

        for i in range(1, num_pages + 1):
            page_hd.wait_for_timeout(300)
            fname = f"{safe_title}_page_{i:03d}.png"
            dest = output_dir / fname

            try:
                if slide_el:
                    slide_el.screenshot(path=str(dest))
                else:
                    page_hd.screenshot(path=str(dest))
                saved.append(dest)
            except Exception as e:
                print(f"  [浏览器] 截图失败 page {i}: {e}")
                try:
                    page_hd.screenshot(path=str(dest))
                    saved.append(dest)
                except Exception:
                    pass

            if i % 10 == 0 or i == num_pages:
                print(f"  [浏览器] 进度: {i}/{num_pages}")

            if i < num_pages:
                page_hd.keyboard.press("ArrowRight")
                page_hd.wait_for_timeout(800)

        if shape_images:
            shapes_dir = output_dir / "shapes"
            shapes_dir.mkdir(exist_ok=True)
            for idx, (h, data) in enumerate(shape_images.items(), 1):
                sp = shapes_dir / f"shape_{idx:03d}_{h[:12]}.png"
                sp.write_bytes(data)
            print(f"  [原图] 额外捕获 {len(shape_images)} 张嵌入图片 → {shapes_dir}")

        ctx_hd.close()
        browser.close()

    print(f"  [浏览器] 截图完成: {len(saved)}/{num_pages} 页")
    return saved


def _find_iframe_url(page: Page) -> str | None:
    """Extract the WebOffice preview iframe URL from the parent page."""
    for frame in page.frames:
        url = frame.url
        if any(x in url for x in ["prvsh.myqcloud.com/office", "preview.myqcloud.com/office"]):
            return url
    iframes = page.query_selector_all("iframe")
    for iframe in iframes:
        src = iframe.get_attribute("src") or ""
        if any(x in src for x in ["prvsh.myqcloud.com", "preview.myqcloud.com", "office/p/"]):
            return src
    return None


def _get_domain(url: str) -> str | None:
    m = re.search(r"https?://([^/]+)", url)
    return m.group(1) if m else None


def _find_slide_element_on_page(page: Page) -> object | None:
    """Find the slide rendering element directly on the page (not iframe)."""
    for sel in (".play_root_container", ".play_root", ".play_svg_container", ".wpp_workspace_container"):
        el = page.query_selector(sel)
        if el:
            box = el.bounding_box()
            if box and box["width"] > 100 and box["height"] > 100:
                return el
    return None


def _extract_shape_hash(url: str) -> str | None:
    m = re.search(r"shapes[/%]2[fF][^/?]+[/%]2[fF]([a-f0-9]+)", url)
    if m:
        return m.group(1)
    m = re.search(r"shapes/[^/]+/([a-f0-9]+)", url)
    if m:
        return m.group(1)
    return None


def _safe(name: str, max_len: int = 80) -> str:
    name = re.sub(r'[<>:"/\\|?*\x00-\x1f]', "_", name).strip(". ")
    return name[:max_len] if name else "untitled"
