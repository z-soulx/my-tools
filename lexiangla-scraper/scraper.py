"""Lexiangla scraper using browser cookies for authentication.

Endpoints discovered:
  - GET /api/v1/docs/{doc_id}          document detail (flat JSON)
  - GET /api/v1/docs/{doc_id}/download  file download (when permitted)
"""

from __future__ import annotations

import json
import re
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlparse

import requests
from markdownify import markdownify as html_to_md


API_BASE = "https://lexiangla.com/api/v1"

URL_PATTERN = re.compile(
    r"lexiangla\.com/(?:teams/([^/]+)/)?docs/([a-f0-9]+)"
)


@dataclass
class DocResult:
    doc_id: str
    title: str
    doc_type: str  # "document" (rich text) or "file" (pptx/docx/xlsx/...)
    meta: dict[str, Any] = field(default_factory=dict)

    html_content: str | None = None
    md_content: str | None = None

    download_url: str | None = None
    downloadable: bool = False
    file_name: str | None = None
    file_ext: str | None = None


def parse_url(url_or_id: str) -> tuple[str | None, str]:
    """Extract (team_code, doc_id) from a lexiangla URL or plain doc_id."""
    m = URL_PATTERN.search(url_or_id)
    if m:
        return m.group(1), m.group(2)
    if re.fullmatch(r"[a-f0-9]{32}", url_or_id):
        return None, url_or_id
    raise ValueError(f"Cannot parse doc_id from: {url_or_id}")


def _parse_cookie_string(cookie_str: str) -> dict[str, str]:
    cookies: dict[str, str] = {}
    for pair in cookie_str.split(";"):
        pair = pair.strip()
        if "=" in pair:
            k, v = pair.split("=", 1)
            cookies[k.strip()] = v.strip()
    return cookies


class LexianglaClient:
    """Scraper client using browser cookies for auth."""

    def __init__(self, cookie_str: str):
        self._session = requests.Session()
        cookies = _parse_cookie_string(cookie_str)
        for k, v in cookies.items():
            self._session.cookies.set(k, v, domain="lexiangla.com")

        xsrf = cookies.get("XSRF-TOKEN", "")
        if xsrf:
            xsrf = unquote(unquote(xsrf))

        self._session.headers.update({
            "User-Agent": (
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:148.0) "
                "Gecko/20100101 Firefox/148.0"
            ),
            "Accept": "application/json, text/plain, */*",
            "X-Auth-Type": "api",
            "X-Requested-With": "XMLHttpRequest",
            "X-XSRF-TOKEN": xsrf,
        })

    def _get(self, url: str, **kwargs: Any) -> requests.Response:
        resp = self._session.get(url, **kwargs)
        resp.raise_for_status()
        return resp

    def _get_json(self, url: str, **kwargs: Any) -> dict:
        return self._get(url, **kwargs).json()

    # ------------------------------------------------------------------
    # Document fetching
    # ------------------------------------------------------------------

    def fetch_doc(self, url_or_id: str) -> DocResult:
        """Fetch a single document by URL or doc_id."""
        _, doc_id = parse_url(url_or_id)
        data = self._get_json(f"{API_BASE}/docs/{doc_id}")
        return self._parse_response(doc_id, data)

    def fetch_raw(self, url_or_id: str) -> dict:
        """Fetch raw API response for debugging."""
        _, doc_id = parse_url(url_or_id)
        return self._get_json(f"{API_BASE}/docs/{doc_id}")

    def _parse_response(self, doc_id: str, data: dict) -> DocResult:
        title = data.get("name") or "untitled"
        target = data.get("target") or {}
        target_model = target.get("model", "")  # "file" or "document"
        file_type = data.get("type", "")  # "pptx", "docx", "document", etc.

        is_file = target_model == "file" or file_type in (
            "pptx", "ppt", "docx", "doc", "xlsx", "xls", "pdf", "zip", "rar",
            "mp4", "mp3", "wav", "jpg", "png", "gif",
        )
        doc_type = "file" if is_file else "document"

        owner = data.get("owner") or target.get("owner") or {}
        team = data.get("team") or target.get("team") or {}
        category = data.get("category") or {}
        breadcrumb = data.get("breadcrumb") or []

        meta = {
            "doc_id": doc_id,
            "title": title,
            "type": file_type,
            "doc_type": doc_type,
            "created_at": data.get("created_at"),
            "updated_at": data.get("updated_at"),
            "read_count": data.get("read_count"),
            "comment_count": data.get("comment_count"),
            "like_count": data.get("like_count"),
            "favorite_count": data.get("favorite_count"),
            "tags": target.get("tags") or [],
            "owner": owner.get("display_name") if isinstance(owner, dict) else None,
            "owner_org": owner.get("organization") if isinstance(owner, dict) else None,
            "team": team.get("name") if isinstance(team, dict) else None,
            "team_code": team.get("code") if isinstance(team, dict) else None,
            "category": category.get("name") if isinstance(category, dict) else None,
            "breadcrumb": [b.get("name") for b in breadcrumb if isinstance(b, dict)],
            "signature": target.get("signature"),
        }

        result = DocResult(
            doc_id=doc_id,
            title=title,
            doc_type=doc_type,
            meta=meta,
        )

        if doc_type == "document":
            content = target.get("content") or data.get("content") or ""
            md_content = target.get("md_content") or ""
            if content:
                result.html_content = content
                result.md_content = md_content or html_to_md(content, heading_style="ATX")
        elif doc_type == "file":
            result.file_name = target.get("name") or title
            result.file_ext = f".{file_type}" if file_type else ""
            result.downloadable = bool(target.get("downloadable"))
            if result.downloadable:
                result.download_url = f"{API_BASE}/docs/{doc_id}/download"

        if target.get("preview_page"):
            meta["preview_pages"] = target["preview_page"]
            meta["preview_status"] = target.get("preview_status")

        return result

    # ------------------------------------------------------------------
    # Download & save
    # ------------------------------------------------------------------

    def download_file(self, url: str, dest: Path) -> Path:
        resp = self._get(url, stream=True, allow_redirects=True)
        dest.parent.mkdir(parents=True, exist_ok=True)
        with open(dest, "wb") as f:
            for chunk in resp.iter_content(chunk_size=8192):
                f.write(chunk)
        return dest

    def try_download(self, doc_id: str, dest: Path) -> Path | None:
        """Try to download a file, return None if not allowed."""
        url = f"{API_BASE}/docs/{doc_id}/download"
        try:
            return self.download_file(url, dest)
        except requests.HTTPError as e:
            if e.response is not None and e.response.status_code == 403:
                return None
            raise

    def save_doc(
        self,
        result: DocResult,
        output_dir: Path,
        *,
        capture_preview: bool = True,
        cookie_str: str | None = None,
        doc_url: str | None = None,
    ) -> list[Path]:
        output_dir.mkdir(parents=True, exist_ok=True)
        safe_title = _safe_filename(result.title)
        saved: list[Path] = []

        meta_path = output_dir / f"{safe_title}.meta.json"
        meta_path.write_text(
            json.dumps(result.meta, ensure_ascii=False, indent=2),
            encoding="utf-8",
        )
        saved.append(meta_path)

        if result.doc_type == "document":
            if result.html_content:
                p = output_dir / f"{safe_title}.html"
                p.write_text(result.html_content, encoding="utf-8")
                saved.append(p)
            if result.md_content:
                p = output_dir / f"{safe_title}.md"
                p.write_text(result.md_content, encoding="utf-8")
                saved.append(p)
        elif result.doc_type == "file":
            ext = result.file_ext or ""
            file_path = output_dir / f"{safe_title}{ext}"
            downloaded = self.try_download(result.doc_id, file_path)
            if downloaded:
                saved.append(downloaded)
            elif capture_preview and cookie_str:
                total_pages = result.meta.get("preview_pages")
                if total_pages and doc_url:
                    try:
                        from preview_capture import capture_preview_images

                        preview_dir = output_dir / f"{safe_title}_pages"
                        images = capture_preview_images(
                            doc_url=doc_url,
                            cookie_str=cookie_str,
                            output_dir=preview_dir,
                            title=result.title,
                            total_pages=total_pages,
                        )
                        saved.extend(images)
                        if images:
                            pdf_path = _images_to_pdf(images, output_dir / f"{safe_title}.pdf")
                            if pdf_path:
                                saved.append(pdf_path)
                    except ImportError:
                        print("  [提示] 安装 playwright 后可截取预览图片: pip install playwright && playwright install chromium")
                    except Exception as e:
                        print(f"  [预览截取失败] {e}")
                else:
                    print(f"  [跳过下载] 文件不允许下载: {result.title}")
            else:
                print(f"  [跳过下载] 文件不允许下载: {result.title}")

        return saved


def _images_to_pdf(images: list[Path], pdf_path: Path) -> Path | None:
    """Merge images into a PDF. Resizes to 1920-wide for a compact, sharp PDF."""
    try:
        from PIL import Image

        sorted_imgs = sorted(images)
        if not sorted_imgs:
            return None

        first = Image.open(sorted_imgs[0])
        orig_w, orig_h = first.size
        first.close()

        target_w = min(orig_w, 1920)
        scale = target_w / orig_w
        target_h = int(orig_h * scale)
        dpi = 150

        print(f"  [PDF] 原图: {orig_w}x{orig_h}, PDF 页面: {target_w}x{target_h}, DPI: {dpi}")

        pdf_imgs: list[Image.Image] = []
        for p in sorted_imgs:
            im = Image.open(p).convert("RGB")
            if im.size != (target_w, target_h):
                im = im.resize((target_w, target_h), Image.LANCZOS)
            pdf_imgs.append(im)

        pdf_imgs[0].save(
            str(pdf_path),
            save_all=True,
            append_images=pdf_imgs[1:],
            resolution=dpi,
        )
        for im in pdf_imgs:
            im.close()

        size_mb = pdf_path.stat().st_size / 1024 / 1024
        print(f"  已合成 PDF: {pdf_path} ({size_mb:.1f} MB)")
        return pdf_path
    except ImportError:
        print("  [提示] 安装 Pillow 后可自动合成 PDF: pip install Pillow")
        return None
    except Exception as e:
        print(f"  [PDF 合成失败] {e}")
        return None


def _safe_filename(name: str, max_len: int = 100) -> str:
    name = re.sub(r'[<>:"/\\|?*\x00-\x1f]', "_", name).strip(". ")
    return name[:max_len] if name else "untitled"
