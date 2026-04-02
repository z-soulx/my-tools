#!/usr/bin/env python
"""CLI for Lexiangla document scraper (cookie-based auth)."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from dotenv import load_dotenv

from scraper import LexianglaClient, parse_url


def get_client(cookie: str | None = None) -> LexianglaClient:
    load_dotenv()
    cookie_str = cookie or os.getenv("LEXIANGLA_COOKIE")

    if not cookie_str:
        print(
            "错误: 未提供 Cookie。请通过 --cookie 参数或 .env 中的 LEXIANGLA_COOKIE 配置。\n"
            "获取方式: 浏览器 F12 → Network → 任意请求 → 复制 Cookie 请求头的值",
            file=sys.stderr,
        )
        sys.exit(1)

    return LexianglaClient(cookie_str)


def cmd_fetch(args: argparse.Namespace) -> None:
    load_dotenv()
    cookie_str = args.cookie or os.getenv("LEXIANGLA_COOKIE") or ""
    client = get_client(args.cookie)
    output_dir = Path(args.output)

    for url in args.urls:
        try:
            print(f"正在获取: {url}")
            result = client.fetch_doc(url)
            print(f"  标题: {result.title}")
            print(f"  类型: {result.doc_type} ({result.meta.get('type', '?')})")
            if result.meta.get("owner"):
                print(f"  作者: {result.meta['owner']}")
            if result.doc_type == "file":
                dl = "可下载" if result.downloadable else "不可下载"
                print(f"  文件: {dl}")
                if result.meta.get("preview_pages"):
                    print(f"  预览: {result.meta['preview_pages']} 页")

            saved = client.save_doc(
                result,
                output_dir,
                capture_preview=not args.no_preview,
                cookie_str=cookie_str,
                doc_url=url,
            )
            for p in saved:
                print(f"  已保存: {p}")
            print()
        except Exception as e:
            print(f"  错误: {e}", file=sys.stderr)
            if not args.continue_on_error:
                sys.exit(1)


def cmd_debug(args: argparse.Namespace) -> None:
    """Fetch raw API response for debugging."""
    client = get_client(args.cookie)
    try:
        data = client.fetch_raw(args.url)
        print(json.dumps(data, ensure_ascii=False, indent=2))
    except Exception as e:
        print(f"错误: {e}", file=sys.stderr)
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="lexiangla-scraper",
        description="抓取腾讯乐享文档（使用浏览器 Cookie 认证）",
    )
    parser.add_argument(
        "--cookie",
        help="浏览器 Cookie 字符串（也可通过 .env 中 LEXIANGLA_COOKIE 配置）",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    p_fetch = subparsers.add_parser("fetch", help="抓取一篇或多篇文档")
    p_fetch.add_argument("urls", nargs="+", help="文档 URL 或 doc_id")
    p_fetch.add_argument("-o", "--output", default="./output", help="输出目录 (默认 ./output)")
    p_fetch.add_argument("-c", "--continue-on-error", action="store_true", help="遇到错误继续处理下一篇")
    p_fetch.add_argument("--no-preview", action="store_true", help="不截取预览图片（跳过 Playwright）")
    p_fetch.set_defaults(func=cmd_fetch)

    p_debug = subparsers.add_parser("debug", help="调试: 查看 API 原始返回")
    p_debug.add_argument("url", help="文档 URL 或 doc_id")
    p_debug.set_defaults(func=cmd_debug)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
