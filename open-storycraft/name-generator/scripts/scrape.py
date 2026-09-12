#!/usr/bin/env python3
"""
Scrape name lists from tekeli.li/onomastikon into data/ txt files.

Usage:
  python3 scrape.py <url> [output_name]
  python3 scrape.py --batch urls.txt

The output name defaults to the last path segment of the URL (e.g. "Male" → male.txt).
All files are saved to the data/ folder next to this script.
"""

from __future__ import annotations

import argparse
import re
import sys
import time
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

DATA_DIR = Path(__file__).parent.parent / "data"


class NameTableParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self._in_td = False
        self.names: list[str] = []

    def handle_starttag(self, tag, attrs):
        if tag == "td":
            self._in_td = True

    def handle_endtag(self, tag):
        if tag == "td":
            self._in_td = False

    def handle_data(self, data):
        if self._in_td:
            name = data.strip()
            if self._is_valid_name(name):
                self.names.append(name)

    @staticmethod
    def _is_valid_name(name: str) -> bool:
        if not name or len(name) < 2:
            return False
        # reject anything with digits, parens, brackets, slashes, or leading punctuation
        import re
        if re.search(r"[\d()\[\]/<>]", name):
            return False
        if name[0] in "-.,;:":
            return False
        # must be mostly letters (allow hyphens, apostrophes, spaces between words)
        if not re.match(r"^[A-Za-z\u00C0-\u024F][A-Za-z\u00C0-\u024F'\- ]*$", name):
            return False
        # reject any multi-word entry
        if len(name.split()) > 1:
            return False
        # reject header-like words
        skip = {"male", "female", "note", "notes", "see", "also", "source", "sources", "name", "names",
                "of", "son", "daughter", "fem", "masc", "gk", "lat", "var"}
        if name.lower() in skip:
            return False
        return True


def fetch(url: str) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return resp.read().decode("utf-8", errors="replace")


def scrape(url: str, output_name: str | None = None) -> Path:
    print(f"Fetching {url} ...")
    html = fetch(url)

    parser = NameTableParser()
    parser.feed(html)

    names = sorted(set(parser.names))
    if not names:
        print(f"  WARNING: no names found at {url}", file=sys.stderr)
        return None

    if output_name is None:
        # derive from URL: last non-empty path segment, strip .html
        segment = [p for p in url.rstrip("/").split("/") if p][-1]
        output_name = re.sub(r"\.html?$", "", segment, flags=re.IGNORECASE)

    output_name = output_name.lower().replace(" ", "-")
    out_path = DATA_DIR / f"{output_name}.txt"
    DATA_DIR.mkdir(exist_ok=True)
    out_path.write_text("\n".join(names) + "\n", encoding="utf-8")
    print(f"  Saved {len(names)} names → {out_path.name}")
    return out_path


def main():
    parser = argparse.ArgumentParser(description="Scrape Onomastikon name lists")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("url", nargs="?", help="Single URL to scrape")
    group.add_argument("--batch", metavar="FILE", help="Text file with one URL per line (optionally: url<TAB>name)")
    parser.add_argument("output_name", nargs="?", help="Output file stem (optional for single URL)")
    args = parser.parse_args()

    if args.url:
        scrape(args.url, args.output_name)
    else:
        lines = Path(args.batch).read_text().splitlines()
        for line in lines:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = line.split("\t", 1)
            url = parts[0].strip()
            name = parts[1].strip() if len(parts) > 1 else None
            scrape(url, name)
            time.sleep(0.5)  # be polite


if __name__ == "__main__":
    main()
