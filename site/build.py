#!/usr/bin/env python3
"""Build the dependency-free Cepheus Trader player documentation site."""

from __future__ import annotations

import argparse
import html
import json
import re
import shutil
import tomllib
from collections import defaultdict
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
CPP_HELP = ROOT / "client" / "src" / "door_help.cpp"
HELP_HEADER = ROOT / "client" / "include" / "ct" / "door_help.hpp"
PLAYER_GUIDE = ROOT / "docs" / "player-guide.md"
SHIP_CATALOG = ROOT / "catalog" / "ships"

PUBLISHED_SHIP_ART = {
    1: (
        "assets/ships/ship-001-hermes.webp",
        "Painted three-quarter view of Hermes, a yellow and blue ten-ton "
        "distributed courier pod with a sealed central cargo module.",
    ),
    2: (
        "assets/ships/ship-002-labyrinth.webp",
        "Painted three-quarter view of Labyrinth, a green and ochre ten-ton "
        "distributed utility pod with a stowed industrial grappling arm.",
    ),
    3: (
        "assets/ships/ship-003-knossos.webp",
        "Painted three-quarter view of Knossos, an ivory, blue, and vermilion "
        "ten-ton distributed passenger pod with four visible cabin windows.",
    ),
    4: (
        "assets/ships/ship-004-minotaur.webp",
        "Painted three-quarter view of Minotaur, an aubergine and orange "
        "ten-ton distributed boarding pod with a grappling arm and docking collar.",
    ),
}

CATEGORY_NAMES = {
    "GettingStarted": "Getting Started",
    "MenusScreens": "Menus & Screens",
    "Concepts": "Core Concepts",
    "Glossary": "Glossary",
}


def slug(text: str) -> str:
    value = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return value or "section"


def cpp_string(token: str) -> str:
    return json.loads(token)


def help_topics() -> list[dict[str, str]]:
    source = CPP_HELP.read_text(encoding="utf-8")
    string = r'"(?:\\.|[^"\\])*"'
    entry = re.compile(
        rf"\{{\s*(?P<title>{string})\s*,\s*(?P<group>{string})\s*,\s*"
        rf"Category::(?P<category>\w+)\s*,\s*(?P<beginner>{string})\s*,\s*"
        rf"(?P<expert>{string})\s*\}}",
        re.DOTALL,
    )
    topics = []
    for match in entry.finditer(source):
        fields = match.groupdict()
        topics.append(
            {
                "title": cpp_string(fields["title"]),
                "group": cpp_string(fields["group"]),
                "category": fields["category"],
                "beginner": cpp_string(fields["beginner"]),
            }
        )

    header = HELP_HEADER.read_text(encoding="utf-8")
    enum = re.search(r"enum class DoorHelpTopic[^\{]*\{(?P<body>.*?)\};", header, re.DOTALL)
    if not enum:
        raise RuntimeError("DoorHelpTopic enum was not found")
    expected = [
        line.strip().rstrip(",")
        for line in enum.group("body").splitlines()
        if line.strip() and line.strip().rstrip(",") != "Count"
    ]
    if len(topics) != len(expected):
        raise RuntimeError(
            f"parsed {len(topics)} help topics, but DoorHelpTopic declares {len(expected)}"
        )
    for topic, enum_name in zip(topics, expected, strict=True):
        topic["id"] = f"help-{slug(enum_name)}"
    return topics


def inline_markdown(text: str) -> str:
    placeholders: list[str] = []

    def preserve_code(match: re.Match[str]) -> str:
        placeholders.append(f"<code>{html.escape(match.group(1))}</code>")
        return f"\x00{len(placeholders) - 1}\x00"

    text = re.sub(r"`([^`]+)`", preserve_code, text)
    text = html.escape(text, quote=False)
    text = re.sub(
        r"\[([^]]+)\]\(([^)]+)\)",
        lambda m: (
            f'<a href="{html.escape(m.group(2), quote=True)}">{m.group(1)}</a>'
        ),
        text,
    )
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    text = re.sub(r"(?<!\*)\*([^*]+)\*(?!\*)", r"<em>\1</em>", text)
    for index, value in enumerate(placeholders):
        text = text.replace(f"\x00{index}\x00", value)
    return text


def markdown_document(markdown: str) -> tuple[str, list[tuple[int, str, str]]]:
    lines = markdown.splitlines()
    rendered: list[str] = []
    headings: list[tuple[int, str, str]] = []
    used_ids: defaultdict[str, int] = defaultdict(int)
    index = 0

    def heading_id(title: str) -> str:
        base = slug(re.sub(r"[`*_]", "", title))
        used_ids[base] += 1
        return base if used_ids[base] == 1 else f"{base}-{used_ids[base]}"

    while index < len(lines):
        line = lines[index]
        if not line.strip():
            index += 1
            continue
        heading = re.match(r"^(#{1,3})\s+(.+)$", line)
        if heading:
            level = len(heading.group(1))
            title = heading.group(2).strip()
            if level == 1:
                index += 1
                continue
            identifier = heading_id(title)
            headings.append((level, re.sub(r"[`*_]", "", title), identifier))
            rendered.append(
                f'<h{level} id="{identifier}">{inline_markdown(title)}'
                f'<a class="heading-anchor" href="#{identifier}" aria-label="Link to this section">#</a>'
                f"</h{level}>"
            )
            index += 1
            continue
        if line.startswith("|") and index + 1 < len(lines) and re.match(
            r"^\|(?:\s*:?-+:?\s*\|)+$", lines[index + 1]
        ):
            headers = [cell.strip() for cell in line.strip("|").split("|")]
            index += 2
            rows = []
            while index < len(lines) and lines[index].startswith("|"):
                rows.append([cell.strip() for cell in lines[index].strip("|").split("|")])
                index += 1
            rendered.append('<div class="table-scroll"><table><thead><tr>')
            rendered.extend(f"<th>{inline_markdown(cell)}</th>" for cell in headers)
            rendered.append("</tr></thead><tbody>")
            for row in rows:
                rendered.append("<tr>")
                rendered.extend(f"<td>{inline_markdown(cell)}</td>" for cell in row)
                rendered.append("</tr>")
            rendered.append("</tbody></table></div>")
            continue
        list_match = re.match(r"^(?P<marker>-|\d+\.)\s+(?P<body>.+)$", line)
        if list_match:
            ordered = list_match.group("marker") != "-"
            tag = "ol" if ordered else "ul"
            rendered.append(f"<{tag}>")
            while index < len(lines):
                item = re.match(r"^(?:-|\d+\.)\s+(.+)$", lines[index])
                if not item:
                    break
                body = item.group(1).strip()
                index += 1
                while (
                    index < len(lines)
                    and lines[index].strip()
                    and not re.match(r"^(?:-|\d+\.)\s+", lines[index])
                    and not re.match(r"^#{1,3}\s+", lines[index])
                ):
                    body += " " + lines[index].strip()
                    index += 1
                rendered.append(f"<li>{inline_markdown(body)}</li>")
                while index < len(lines) and not lines[index].strip():
                    index += 1
                if index >= len(lines) or not re.match(r"^(?:-|\d+\.)\s+", lines[index]):
                    break
            rendered.append(f"</{tag}>")
            continue

        paragraph = [line.strip()]
        index += 1
        while (
            index < len(lines)
            and lines[index].strip()
            and not re.match(r"^#{1,3}\s+", lines[index])
            and not re.match(r"^(?:-|\d+\.)\s+", lines[index])
            and not lines[index].startswith("|")
        ):
            paragraph.append(lines[index].strip())
            index += 1
        rendered.append(f"<p>{inline_markdown(' '.join(paragraph))}</p>")

    return "\n".join(rendered), headings


def page_shell(title: str, description: str, current: str, body: str) -> str:
    nav = []
    for key, label, href in (
        ("home", "Home", "index.html"),
        ("catalog", "Ship Catalog", "ships.html"),
        ("reference", "Player Reference", "reference.html"),
        ("help", "Beginner Help", "beginner-help.html"),
    ):
        active = ' aria-current="page"' if current == key else ""
        nav.append(f'<a href="{href}"{active}>{label}</a>')
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="theme-color" content="#171b20">
  <meta name="description" content="{html.escape(description, quote=True)}">
  <title>{html.escape(title)} · Cepheus Trader</title>
  <link rel="stylesheet" href="assets/site.css">
  <script src="assets/site.js" defer></script>
</head>
<body data-page="{current}">
  <a class="skip-link" href="#main">Skip to content</a>
  <div class="page-grid" aria-hidden="true"></div>
  <header class="site-header">
    <a class="wordmark" href="index.html" aria-label="Cepheus Trader home">
      <span class="wordmark-mark">CT</span>
      <span><strong>Cepheus Trader</strong><small>Interstellar command service</small></span>
    </a>
    <button class="nav-toggle" type="button" aria-expanded="false" aria-controls="site-nav">Menu</button>
    <nav id="site-nav" class="site-nav" aria-label="Primary">{''.join(nav)}</nav>
  </header>
  <main id="main">{body}</main>
  <footer class="site-footer">
    <div><span class="footer-signal" aria-hidden="true"></span> Transmission continues while you are away.</div>
    <div class="footer-links">
      <a href="https://github.com/RealDeuce/ct">Source</a>
      <a href="https://github.com/RealDeuce/ct/issues">Report an issue</a>
      <a href="https://github.com/RealDeuce/ct/blob/main/OPEN_GAME_LICENSE.md">Game license</a>
    </div>
  </footer>
</body>
</html>
"""


def landing_page() -> str:
    body = """
<section class="hero">
  <div class="hero-copy">
    <p class="eyebrow">Trade // Command // Survive</p>
    <h1>Someone else is already<br><em>changing the map.</em></h1>
    <p class="hero-lead">Cepheus Trader is a persistent multiplayer space-trading game played through participating BBSs. Command a captain, a living crew, and a starship in a universe where distance, debt, law, and delayed information matter.</p>
    <div class="hero-actions">
      <a class="button button-primary" href="beginner-help.html#help-orientation">Begin your first watch</a>
      <a class="button button-secondary" href="reference.html">Open player reference</a>
    </div>
    <ul class="signal-strip" aria-label="Game attributes">
      <li>Shared universe</li><li>Persistent time</li><li>BBS multiplayer</li><li>Field alpha</li>
    </ul>
  </div>
  <div class="system-plot" role="img" aria-label="Stylized orbital plot with a ship leaving a ringed world">
    <span class="plot-label plot-label-a">MARCHES / 77</span>
    <span class="plot-label plot-label-b">J-2 VECTOR</span>
    <span class="orbit orbit-one"></span>
    <span class="orbit orbit-two"></span>
    <span class="planet"></span>
    <span class="moon"></span>
    <span class="vector"></span>
    <span class="ship-glyph">◆</span>
  </div>
</section>

<section class="dispatch" aria-labelledby="dispatch-title">
  <p class="section-index">01 / Incoming dispatch</p>
  <div class="dispatch-grid">
    <h2 id="dispatch-title">The papers on the desk carry your name.</h2>
    <div>
      <p>Beyond the port glass, ships are lifting for worlds whose news has not arrived here yet. Some carry cargo, some carry warrants, and some carry guns. One of them can become your command.</p>
      <p>You are creating a captain, not solving an entrance examination. Every starting command is playable. Choose the kind of problems you want: trade and debt, licensed private force, or naval duty and authority.</p>
    </div>
  </div>
</section>

<section class="career-section" aria-labelledby="career-title">
  <div class="section-heading">
    <div><p class="section-index">02 / Choose a commission</p><h2 id="career-title">Three ways into trouble</h2></div>
    <p>Career changes ownership, authority, obligations, acceptable conduct, and the terms for leaving. It is not a difficulty selector.</p>
  </div>
  <div class="career-grid">
    <article class="career-card trader"><span class="card-number">A</span><h3>Trader</h3><p>Independent commerce, real equity, secured debt, finite markets, and the freedom to decide which risks are worth taking.</p><a href="reference.html#choose-a-starting-offer">Read the terms <span aria-hidden="true">→</span></a></article>
    <article class="career-card privateer"><span class="card-number">B</span><h3>Privateer</h3><p>An armed charter, sponsor-owned hull, prize rights, operating credit, and legal authority that ends exactly where its terms do.</p><a href="reference.html#sponsor-owned-ships-and-arrears">Know the obligation <span aria-hidden="true">→</span></a></article>
    <article class="career-card navy"><span class="card-number">C</span><h3>Navy</h3><p>Public property under your command: rank, orders, service pay, restricted accounts, and consequences that travel by mail.</p><a href="reference.html#naval-service-accounts">Review service accounts <span aria-hidden="true">→</span></a></article>
  </div>
</section>

<section class="principles" aria-labelledby="principles-title">
  <div class="principles-title"><p class="section-index">03 / Operating principles</p><h2 id="principles-title">No omniscient map.<br>No paused universe.</h2></div>
  <div class="principle-list">
    <article><span>01</span><div><h3>Information has a location</h3><p>News, contracts, market observations, warrants, and replies travel. Two honest captains can know different versions of the same event.</p></div></article>
    <article><span>02</span><div><h3>Your ship is a place</h3><p>Crew, fuel, ammunition, provisions, cargo, prisoners, and damage remain attached to physical vessels—not abstract menu inventories.</p></div></article>
    <article><span>03</span><div><h3>Promises outlive logoff</h3><p>Flight plans continue, deadlines approach, wages come due, and other captains act while you are back on the BBS.</p></div></article>
    <article><span>04</span><div><h3>Authority is specific</h3><p>A weapon gives capability. A commission, warrant, order, or local law determines whether using it is privateering, enforcement, or piracy.</p></div></article>
  </div>
</section>

<section class="first-watch" aria-labelledby="watch-title">
  <div>
    <p class="section-index">04 / First watch checklist</p>
    <h2 id="watch-title">Look before you launch.</h2>
    <p>A cheap, well-crewed, fuelled ship with a feasible plan is safer than a grand vessel sent out on an assumption.</p>
  </div>
  <ol>
    <li><span>U</span> Inspect Crew and Ship Management.</li>
    <li><span>J</span> Read jobs without accepting blind obligations.</li>
    <li><span>C</span> Compare cargo, cash, and free hold space.</li>
    <li><span>F</span> Verify fuel, provisions, and ammunition.</li>
    <li><span>K</span> Read the destination dossier and plot a course.</li>
    <li><span>D</span> Preview every warning before filing departure.</li>
  </ol>
  <a class="text-link" href="reference.html#a-good-first-session">Walk through the complete first session <span aria-hidden="true">→</span></a>
</section>

<section class="resource-callout" aria-labelledby="resources-title">
  <div><p class="section-index">05 / Captain's library</p><h2 id="resources-title">The manual travels with you.</h2></div>
  <a href="ships.html"><strong>Ship Catalog</strong><span>Recognition plates and mechanical summaries for documented vessel families.</span><b aria-hidden="true">↗</b></a>
  <a href="reference.html"><strong>Player Reference</strong><span>Complete controls, systems, operations, travel, combat, and practical advice.</span><b aria-hidden="true">↗</b></a>
  <a href="beginner-help.html"><strong>Beginner Help</strong><span>The same conceptual help available from every <kbd>?</kbd> prompt in the door.</span><b aria-hidden="true">↗</b></a>
</section>
"""
    return page_shell(
        "Persistent space trade through the BBS",
        "The introduction and player help site for Cepheus Trader, a persistent multiplayer BBS space-trading game.",
        "home",
        body,
    )


def display_term(value: str) -> str:
    return value.replace("-", " ").title()


def format_tons(millitons: int) -> str:
    tons = millitons / 1000
    value = f"{tons:,.3f}".rstrip("0").rstrip(".")
    return f"{value} ton{'s' if tons != 1 else ''}"


def catalog_records() -> list[dict[str, object]]:
    names = tomllib.loads((SHIP_CATALOG / "names.toml").read_text(encoding="utf-8"))
    family_names = {
        entry["family_id"]: entry["display_name"] for entry in names["family_name"]
    }
    path_names = {entry["path_id"]: entry for entry in names["path_name"]}
    records: list[dict[str, object]] = []
    for catalog_id, (art_path, art_alt) in PUBLISHED_SHIP_ART.items():
        source = SHIP_CATALOG / f"ship-{catalog_id}.toml"
        data = tomllib.loads(source.read_text(encoding="utf-8"))
        catalog = data["catalog"]
        if catalog["catalog_id"] != catalog_id:
            raise RuntimeError(f"catalog ID mismatch in {source}")
        if not (SITE / art_path).is_file():
            raise RuntimeError(f"missing published ship art: {art_path}")
        hull_match = re.fullmatch(r"(?:small|ship)-(\d+)", data["hull"]["id"])
        if not hull_match:
            raise RuntimeError(f"unknown hull ID in {source}: {data['hull']['id']}")
        equipment = []
        for item in data.get("equipment", []):
            quantity = item.get("quantity", 1)
            if item["id"] == "grappling-arm":
                equipment.append("Grappling arm")
            elif item["id"] == "acceleration-seat":
                equipment.append(f"{quantity} passenger seats")
        if data.get("airlocks", 0):
            equipment.append("Pressure airlock")
        if not equipment:
            equipment.append("Priority cargo module")
        crew = sum(item["quantity"] for item in data.get("crew", []))
        records.append(
            {
                "catalog_id": catalog_id,
                "tag": catalog["tag"],
                "name": catalog["display_name"],
                "role": display_term(catalog["primary_role"]),
                "description": catalog["description_paragraphs"],
                "family_id": catalog["family_id"],
                "family_name": family_names[catalog["family_id"]],
                "path_id": catalog["upgrade_path_id"],
                "path_name": path_names[catalog["upgrade_path_id"]]["display_name"],
                "yard_name": path_names[catalog["upgrade_path_id"]]["manufacturer_name"],
                "tons": int(hull_match.group(1)),
                "configuration": display_term(data["hull"]["configuration"]),
                "crew": crew,
                "endurance": data["fuel"]["power_plant_weeks"],
                "cargo": format_tons(data.get("cargo_millitons", 0)),
                "equipment": " · ".join(equipment),
                "art_path": art_path,
                "art_alt": art_alt,
            }
        )
    return records


def ship_catalog_page() -> str:
    records = catalog_records()
    cards = []
    for ship in records:
        paragraphs = "".join(f"<p>{html.escape(text)}</p>" for text in ship["description"])
        weeks = ship["endurance"]
        cards.append(
            f"""
<article class="ship-card path-{ship['path_id']}" id="{ship['tag']}">
  <figure class="ship-plate">
    <img src="{ship['art_path']}" alt="{html.escape(ship['art_alt'], quote=True)}" width="1536" height="1024" loading="lazy">
    <figcaption>
      <span>{ship['tag']} / Family {ship['family_id']:03}</span>
      <span class="ship-scale">9.6 m overall</span>
    </figcaption>
  </figure>
  <div class="ship-card-body">
    <p class="ship-path">Path {ship['path_id']:02} / {html.escape(ship['path_name'])}</p>
    <div class="ship-title-row"><h2>{html.escape(ship['name'])}</h2><span>{html.escape(ship['role'])}</span></div>
    <dl class="ship-specs">
      <div><dt>Displacement</dt><dd>{ship['tons']} tons</dd></div>
      <div><dt>Configuration</dt><dd>{ship['configuration']}</dd></div>
      <div><dt>Crew</dt><dd>{ship['crew']}</dd></div>
      <div><dt>Endurance</dt><dd>{weeks} week{'s' if weeks != 1 else ''}</dd></div>
      <div><dt>Cargo</dt><dd>{ship['cargo']}</dd></div>
      <div><dt>Armament</dt><dd>Unarmed</dd></div>
    </dl>
    <p class="ship-fit"><span>Recognized fit</span>{html.escape(ship['equipment'])}</p>
    <div class="ship-description">{paragraphs}</div>
    <p class="ship-yard">Constructed in the design language of <strong>{html.escape(ship['yard_name'])}</strong>.</p>
  </div>
</article>"""
        )
    body = f"""
<header class="document-hero catalog-hero">
  <p class="eyebrow">Captain's library // SC-01</p>
  <h1>Ship Catalog</h1>
  <p>Field-recognition plates and working summaries for canonical vessels encountered across the shared universe. Illustration proceeds by complete hull family so related ships remain visibly related.</p>
  <dl class="catalog-overview" aria-label="Catalog publication status">
    <div><dt>Plates issued</dt><dd>{len(records):02}</dd></div>
    <div><dt>Complete families</dt><dd>01</dd></div>
    <div><dt>Catalog designs</dt><dd>213</dd></div>
    <div><dt>Current volume</dt><dd>10 displacement tons</dd></div>
  </dl>
</header>
<section class="catalog-family" aria-labelledby="daedalus-title">
  <header class="catalog-family-header">
    <div><p class="section-index">Family 001 / Auxiliary craft</p><h2 id="daedalus-title">Daedalus work pods</h2></div>
    <div class="family-brief">
      <p>Four local craft share one ten-ton distributed chassis: a two-seat faceted cockpit, open structural spine, replaceable mission cradle, and paired drive drums.</p>
      <ul aria-label="Family characteristics"><li>10 displacement tons</li><li>Distributed hull</li><li>Two crew</li><li>Non-Jump</li><li>Unarmed</li></ul>
    </div>
  </header>
  <div class="catalog-grid">{''.join(cards)}</div>
  <p class="catalog-source"><span>Registry note</span> Mechanics and descriptions are read from the active ship records. Exterior dimensions, component placement, and illustrations are original canonical art decisions recorded in the Daedalus visual manifest.</p>
</section>
"""
    return page_shell(
        "Ship Catalog",
        "Illustrated player-facing ship catalog for Cepheus Trader, beginning with the complete Daedalus work-pod family.",
        "catalog",
        body,
    )


def reference_page() -> str:
    content, headings = markdown_document(PLAYER_GUIDE.read_text(encoding="utf-8"))
    toc = []
    for level, title, identifier in headings:
        css = "toc-sub" if level == 3 else "toc-main"
        toc.append(f'<a class="{css}" href="#{identifier}">{html.escape(title)}</a>')
    body = f"""
<header class="document-hero">
  <p class="eyebrow">Captain's library // PR-01</p>
  <h1>Player Reference</h1>
  <p>The complete operational guide: from the first keypress to trade, finance, fleet command, delayed mail, travel, and combat.</p>
</header>
<div class="document-layout">
  <aside class="document-toc" aria-label="Player reference contents">
    <div class="toc-title">Reference index</div>
    <nav>{''.join(toc)}</nav>
  </aside>
  <article class="prose document-content">
    <div class="source-note"><span>Live document</span> Generated from the player guide shipped with the game.</div>
    {content}
  </article>
</div>
"""
    return page_shell(
        "Player Reference",
        "Complete player documentation for Cepheus Trader, including controls, trade, travel, finance, mail, law, and combat.",
        "reference",
        body,
    )


def help_paragraphs(body: str) -> str:
    return "".join(
        f"<p>{html.escape(paragraph)}</p>"
        for paragraph in body.split("\n\n")
        if paragraph.strip()
    )


def beginner_help_page(topics: list[dict[str, str]]) -> str:
    by_category: defaultdict[str, list[dict[str, str]]] = defaultdict(list)
    for topic in topics:
        by_category[topic["category"]].append(topic)

    sections = []
    category_nav = []
    for category_key, category_name in CATEGORY_NAMES.items():
        category_topics = by_category[category_key]
        category_id = f"category-{slug(category_name)}"
        category_nav.append(
            f'<a href="#{category_id}"><span>{len(category_topics):02}</span>{category_name}</a>'
        )
        groups: defaultdict[str, list[dict[str, str]]] = defaultdict(list)
        for topic in category_topics:
            groups[topic["group"]].append(topic)
        group_html = []
        for group, items in groups.items():
            group_html.append(
                f'<div class="help-group" data-help-group><h3>{html.escape(group)}</h3>'
            )
            for item in items:
                searchable = " ".join((item["title"], item["group"], item["beginner"]))
                group_html.append(
                    f'<article class="help-topic" id="{item["id"]}" data-help-topic '
                    f'data-search="{html.escape(searchable.lower(), quote=True)}">'
                    f'<div class="help-topic-heading"><span>{html.escape(group)}</span>'
                    f'<h4>{html.escape(item["title"])}</h4>'
                    f'<a href="#{item["id"]}" aria-label="Link to {html.escape(item["title"], quote=True)}">#</a></div>'
                    f'<div class="help-copy">{help_paragraphs(item["beginner"])}</div></article>'
                )
            group_html.append("</div>")
        sections.append(
            f'<section class="help-category" id="{category_id}" data-help-category>'
            f'<div class="help-category-heading"><p class="section-index">{len(category_topics):02} topics</p>'
            f'<h2>{html.escape(category_name)}</h2></div>{"".join(group_html)}</section>'
        )

    body = f"""
<header class="document-hero help-hero">
  <p class="eyebrow">Captain's library // BH-{len(topics):02}</p>
  <h1>Beginner Help</h1>
  <p>The full beginner explanation behind every <kbd>?</kbd> prompt in the game. Search by screen, system, or unfamiliar term.</p>
  <div class="help-search">
    <label for="help-query">Search all {len(topics)} topics</label>
    <div><span aria-hidden="true">⌕</span><input id="help-query" type="search" autocomplete="off" placeholder="Try “fuel”, “warrant”, or “crew”…"><kbd>/</kbd></div>
    <p id="help-results" role="status" aria-live="polite">Showing all {len(topics)} topics.</p>
  </div>
</header>
<div class="help-layout">
  <aside class="help-index"><div class="toc-title">Help channels</div><nav>{''.join(category_nav)}</nav></aside>
  <div class="help-content">
    <div class="source-note"><span>In-game text</span> Generated directly from the door's Beginner Help source.</div>
    <div id="no-help-results" class="no-results" hidden><strong>No matching dispatch.</strong><p>Try a shorter term or browse a help channel.</p></div>
    {''.join(sections)}
  </div>
</div>
"""
    return page_shell(
        "Beginner Help",
        "Search all beginner help topics from the Cepheus Trader BBS door.",
        "help",
        body,
    )


class SiteDocumentParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: list[str] = []
        self.hrefs: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if values.get("id"):
            self.ids.append(values["id"] or "")
        if tag == "a" and values.get("href"):
            self.hrefs.append(values["href"] or "")


def validate_output(output: Path) -> None:
    documents: dict[str, SiteDocumentParser] = {}
    for path in output.glob("*.html"):
        parser = SiteDocumentParser()
        parser.feed(path.read_text(encoding="utf-8"))
        duplicates = {value for value in parser.ids if parser.ids.count(value) > 1}
        if duplicates:
            raise RuntimeError(f"duplicate IDs in {path.name}: {sorted(duplicates)}")
        documents[path.name] = parser
    for name, parser in documents.items():
        for href in parser.hrefs:
            parsed = urlsplit(href)
            if parsed.scheme or parsed.netloc or href.startswith("mailto:"):
                continue
            target_name = parsed.path or name
            if target_name not in documents:
                raise RuntimeError(f"{name} links to missing page {target_name}")
            if parsed.fragment and parsed.fragment not in documents[target_name].ids:
                raise RuntimeError(
                    f"{name} links to missing anchor {target_name}#{parsed.fragment}"
                )


def build(output: Path) -> None:
    output = output.resolve()
    if output == ROOT or ROOT not in output.parents:
        raise RuntimeError("output must be a directory inside the repository")
    if output.exists():
        shutil.rmtree(output)
    (output / "assets").mkdir(parents=True)
    topics = help_topics()
    pages = {
        "index.html": landing_page(),
        "ships.html": ship_catalog_page(),
        "reference.html": reference_page(),
        "beginner-help.html": beginner_help_page(topics),
    }
    for name, content in pages.items():
        (output / name).write_text(content, encoding="utf-8")
    for asset in (SITE / "assets").rglob("*"):
        if not asset.is_file():
            continue
        destination = output / "assets" / asset.relative_to(SITE / "assets")
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(asset, destination)
    (output / ".nojekyll").write_text("", encoding="utf-8")
    validate_output(output)
    print(f"Built {len(pages)} pages with {len(topics)} help topics in {output}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, default=SITE / "_site")
    arguments = parser.parse_args()
    build(arguments.output)


if __name__ == "__main__":
    main()
