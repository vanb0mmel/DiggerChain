"""Render public website configuration, the illustrated book, and machine-readable copy."""

import html
from html.parser import HTMLParser
import json
from pathlib import Path
import re
from urllib.parse import urlsplit


SOURCE = Path(__file__).resolve().parents[1] / "website"


def download_markup(release):
    rows = []
    for item in release["downloads"]:
        if bool(item["url"]) != bool(item["checksum"]):
            raise ValueError("A download and its checksum must be supplied together.")
        links = []
        for name, label in (("url", "Download ZIP"), ("checksum", "SHA-256")):
            url = item[name]
            if url is None:
                continue
            parsed = urlsplit(url)
            local = re.fullmatch(r"\./downloads/[A-Za-z0-9._-]+", url)
            remote = parsed.scheme == "https" and parsed.hostname and not parsed.username and not parsed.password
            if not (local or remote):
                raise ValueError("Download links must use HTTPS or a file within ./downloads/.")
            links.append(f'<a href="{html.escape(url, quote=True)}">{label}</a>')
        status = " · ".join(links) if links else "Coming Soon"
        rows.append(f'<div><dt>{html.escape(item["platform"])}</dt><dd>{status}</dd></div>')
    return '<dl class="controls-list">' + "".join(rows) + '</dl>'


def download_markdown(release):
    rows = []
    for item in release["downloads"]:
        status = f'[Download ZIP]({item["url"]}) · [SHA-256]({item["checksum"]})' if item["url"] else "Coming Soon"
        rows.append(f'- {item["platform"]}: {status}')
    return f'Alpha version: {release["version"]}.\n\n' + "\n".join(rows)


def lore_markup(lore):
    pages = []
    for index, page in enumerate(lore["pages"]):
        scene = lore["scenes"][page["scene"]]
        hidden = "hidden" if index else ""
        paragraphs = "".join(f"<p>{html.escape(text)}</p>" for text in page["paragraphs"])
        pages.append(
            f'<article class="book-page" data-reader-page data-slug="{html.escape(page["slug"])}" {hidden}>'
            f'<figure class="book-art"><img src="./assets/lore/{html.escape(scene["art"])}" alt="{html.escape(scene["alt"])}" loading="lazy" decoding="async" width="{scene["width"]}" height="{scene["height"]}"></figure>'
            f'<div class="book-copy"><h2 tabindex="-1">{html.escape(page["title"])}</h2>{paragraphs}</div></article>'
        )
    return "\n".join(pages)


def phase_navigation(roadmap):
    return "\n".join(
        f'<a href="#roadmap/{html.escape(phase["slug"])}" aria-label="Phase {phase["numeral"]}: {html.escape(phase["label"], quote=True)}"><span class="phase-index">{phase["numeral"]}</span><span class="phase-label">{html.escape(phase["label"])}</span></a>'
        for phase in roadmap["phases"]
    )


def phase_markup(roadmap):
    pages = []
    for index, phase in enumerate(roadmap["phases"]):
        hidden = "hidden" if index else ""
        steps = "".join(f"<li>{html.escape(step)}</li>" for step in phase["steps"])
        pages.append(
            f'<article class="phase-page" data-reader-page data-slug="{html.escape(phase["slug"])}" {hidden}>'
            f'<h2 tabindex="-1">{html.escape(phase["title"])}</h2><p class="phase-status">Phase {phase["numeral"]} · {html.escape(phase["status"])}</p>'
            f'<p>{html.escape(phase["summary"])}</p><ul>{steps}</ul></article>'
        )
    return "\n".join(pages)


def lore_markdown(lore):
    parts = [f'# {lore["title"]}', "This is the fictional premise, not a description of the alpha's level count or win conditions."]
    for page in lore["pages"]:
        parts.extend([f'## {page["title"]}', *page["paragraphs"]])
    return "\n\n".join(parts) + "\n"


def roadmap_markdown(roadmap):
    parts = [f'# {roadmap["title"]}', "Later stages depend on the work before them. Direct DCG and RWA-token rewards are planned for Phase III, not alpha or beta."]
    for phase in roadmap["phases"]:
        parts.extend([f'## Phase {phase["numeral"]}: {phase["label"]}', phase["summary"], "\n".join("- " + step for step in phase["steps"])])
    return "\n\n".join(parts) + "\n"

def destination(name, label, url):
    icon = f'<svg viewBox="0 0 24 24" aria-hidden="true"><use href="./assets/icons.svg#{name}"/></svg>'
    if name == "pons":
        icon = '<span class="pons-icon" aria-hidden="true"></span>'
    content = f'{icon}<span class="destination-name">{label}</span><span class="destination-status">'
    if url:
        return f'<a class="destination" href="{html.escape(url, quote=True)}" target="_blank" rel="noopener noreferrer">{content}Open link</span></a>'
    return f'<button class="destination" type="button" disabled>{content}Coming Soon</span></button>'


class GuideMarkdown(HTMLParser):
    """Export the actual guide copy instead of maintaining a second set of game rules."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.depth = 0
        self.parts = []
        self.pre = False
        self.link = None
        self.ignore = 0

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if not self.depth:
            if "guide-copy" not in attrs.get("class", "").split():
                return
            self.depth = 1
        elif tag not in {"br", "img", "hr", "input"}:
            self.depth += 1
        if self.ignore:
            self.ignore += tag not in {"br", "img", "hr", "input"}
            return
        if tag in {"button", "svg"}:
            self.ignore = 1
        elif tag in {"h2", "h3"}:
            self.parts.append("\n\n" + ("## " if tag == "h2" else "### "))
        elif tag in {"p", "section", "div", "dl"}:
            self.parts.append("\n\n")
        elif tag == "dt":
            self.parts.append("\n- ")
        elif tag == "dd":
            self.parts.append(": ")
        elif tag == "pre":
            self.parts.append("\n\n```sh\n")
            self.pre = True
        elif tag == "br":
            self.parts.append("\n")
        elif tag == "code" and not self.pre:
            self.parts.append(" `")
        elif tag == "a":
            self.link = attrs.get("href")
            self.parts.append("[")

    def handle_endtag(self, tag):
        if not self.depth:
            return
        self.depth -= 1
        if self.ignore:
            self.ignore -= 1
            return
        if tag == "pre":
            self.parts.append("\n```\n\n")
            self.pre = False
        elif tag == "a":
            url = self.link or ""
            if url.startswith("#"):
                url = "./docs.md" if url.startswith("#docs") else "./index.html" + url
            self.parts.append(f"]({url})")
            self.link = None
        elif tag == "code" and not self.pre:
            self.parts.append("` ")
        elif tag in {"h2", "h3", "p", "section", "dl"}:
            self.parts.append("\n\n")

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if tag not in {"br", "img", "hr", "input"}:
            self.handle_endtag(tag)

    def handle_data(self, data):
        if self.depth and not self.ignore:
            if not self.pre and data.isspace() and "\n" in data:
                return
            self.parts.append(data if self.pre else re.sub(r"\s+", " ", data))

    def text(self):
        text = re.sub(r"[ \t]+\n", "\n", "".join(self.parts))
        return re.sub(r"\n{3,}", "\n\n", text).strip()


def render_site(output):
    config = json.loads((SOURCE / "site.json").read_text(encoding="utf-8"))
    token = config["token"]
    if token["symbol"] != "DCG" or token["chainId"] != 4663:
        raise ValueError("The website token must be DCG on Robinhood Chain 4663.")
    address = token["contractAddress"]
    if address is not None and not re.fullmatch(r"0x[0-9a-fA-F]{40}", address):
        raise ValueError("Set contractAddress to null or a complete 0x-prefixed EVM address.")
    for url in config["links"].values():
        if url is not None:
            parsed = urlsplit(url)
            if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password:
                raise ValueError("Public links must be complete HTTPS URLs or null.")
    lore = json.loads((SOURCE / "content/lore.json").read_text(encoding="utf-8"))
    roadmap = json.loads((SOURCE / "content/roadmap.json").read_text(encoding="utf-8"))
    values = {
        "DOWNLOADS": download_markup(config["release"]),
        "LORE_PAGES": lore_markup(lore),
        "PHASE_NAV": phase_navigation(roadmap),
        "PHASE_PAGES": phase_markup(roadmap),
        "WORDMARK": (SOURCE / "assets/wordmark.svg").read_text(encoding="utf-8"),
        "TOKEN_ADDRESS": html.escape(address or "Coming Soon"),
        "COPY_DISABLED": "" if address else "disabled",
        "PROJECT_LINKS": "\n".join([
            destination("x", "X / Twitter", config["links"]["x"]),
            destination("pons", "PonsFamily", config["links"]["ponsFamily"]),
            destination("github", "GitHub", config["links"]["github"]),
            destination("robot", "LLMs", "./llms.txt"),
        ]),
    }
    page = (SOURCE / "index.html").read_text(encoding="utf-8")
    page = re.sub(r"\{\{\s*([A-Z_]+)\s*\}\}", lambda match: values[match[1]], page)
    if "{{" in page:
        raise ValueError("The website contains an unresolved template value.")
    (output / "index.html").write_text(page, encoding="utf-8")
    guide = GuideMarkdown()
    guide.feed(page)
    docs = "# DiggerChain player guide\n\n" + guide.text() + "\n"
    (output / "docs.md").write_text(docs, encoding="utf-8")
    machine_values = {
        "DOWNLOADS": download_markdown(config["release"]),
        "TOKEN_ADDRESS": address or "Coming Soon — no official token contract address is published yet.",
        "GUIDE": docs,
        "LORE": lore_markdown(lore),
        "ROADMAP": roadmap_markdown(roadmap),
        "EXTERNAL_LINKS": "\n".join(f"- {name}: {url or 'Coming Soon'}" for name, url in config["links"].items()),
    }
    (output / "lore.md").write_text(lore_markdown(lore), encoding="utf-8")
    (output / "roadmap.md").write_text(roadmap_markdown(roadmap), encoding="utf-8")
    for name in ("llms.txt", "llms-full.txt"):
        text = (SOURCE / name).read_text(encoding="utf-8")
        for key, value in machine_values.items():
            text = text.replace("{{" + key + "}}", value)
        (output / name).write_text(text, encoding="utf-8")
    (output / "site.json").write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
