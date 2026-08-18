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
SHIPBUILDING_CORE = ROOT / "catalog" / "shipbuilding" / "ce-core.toml"

PUBLISHED_SHIP_ART = {
    1: (
        "assets/ships/ship-001-hermes.webp",
        "Painted three-quarter view of Hermes, a yellow and blue ten-ton "
        "distributed courier pod with a sealed central cargo module.",
        9.6,
    ),
    2: (
        "assets/ships/ship-002-labyrinth.webp",
        "Painted three-quarter view of Labyrinth, a green and ochre ten-ton "
        "distributed utility pod with a stowed industrial grappling arm.",
        9.6,
    ),
    3: (
        "assets/ships/ship-003-knossos.webp",
        "Painted three-quarter view of Knossos, an ivory, blue, and vermilion "
        "ten-ton distributed passenger pod with four visible cabin windows.",
        9.6,
    ),
    4: (
        "assets/ships/ship-004-minotaur.webp",
        "Painted three-quarter view of Minotaur, an aubergine and orange "
        "ten-ton distributed boarding pod with a grappling arm and docking collar.",
        9.6,
    ),
    5: (
        "assets/ships/family-005-charon.webp",
        "Painted three-quarter view of Charon, an ivory, vermilion, and blue "
        "ten-ton streamlined armored passenger launch with three cabin windows.",
        12.5,
    ),
    6: (
        "assets/ships/ship-006-aeolus.webp",
        "Painted three-quarter view of Aeolus, an ivory, blue, and vermilion "
        "twenty-ton streamlined utility launch with six cabin windows.",
        18.5,
    ),
    7: (
        "assets/ships/family-007-caduceus-venture.webp",
        "Painted three-quarter view of Caduceus, a sunflower and cobalt "
        "twenty-ton armored fast launch with a closed dorsal hardpoint.",
        17.5,
    ),
    8: (
        "assets/ships/ship-008-argus.webp",
        "Painted three-quarter view of Argus, a sunflower and cobalt "
        "twenty-ton armored launch with one dorsal beam-laser turret.",
        17.5,
    ),
    9: (
        "assets/ships/ship-009-icarus-i.webp",
        "Painted three-quarter view of Icarus I, a navy, ivory, and red "
        "ten-ton wingless carrier interceptor with a shuttered missile turret.",
        11.8,
    ),
    11: (
        "assets/ships/ship-011-icarus-ii.webp",
        "Painted three-quarter view of Icarus II, a navy, ivory, and red "
        "ten-ton wingless carrier interceptor with doubled armor bands.",
        11.8,
    ),
    12: (
        "assets/ships/ship-012-skuld.webp",
        "Painted three-quarter view of Skuld, an ultramarine and violet "
        "ten-ton arrowhead missile fighter with a closed yellow launch shutter.",
        12.4,
    ),
    15: (
        "assets/ships/ship-015-sigrun.webp",
        "Painted three-quarter view of Sigrun, a navy and ivory ten-ton "
        "arrowhead fighter with one fixed dorsal beam-laser emitter.",
        12.4,
    ),
    17: (
        "assets/ships/ship-017-jason.webp",
        "Painted three-quarter view of Jason, an emerald and cream thirty-ton "
        "armored transport with four cabin windows and a beam-laser turret.",
        21.0,
    ),
    18: (
        "assets/ships/ship-018-wayfarer-utility.webp",
        "Painted three-quarter view of Wayfarer Utility, a yellow and cobalt "
        "thirty-ton utility boat with a fixed dorsal beam-laser emitter.",
        18.8,
    ),
    19: (
        "assets/ships/ship-019-medea.webp",
        "Painted three-quarter view of Medea, an emerald and cream thirty-ton "
        "boarding transport with six cabin windows and airlock handholds.",
        21.0,
    ),
    20: (
        "assets/ships/ship-020-proteus-cargo.webp",
        "Painted three-quarter view of Proteus Cargo, an ivory, vermilion, and "
        "blue fifty-ton modular lighter with a broad two-leaf cargo door.",
        26.0,
    ),
    21: (
        "assets/ships/ship-021-proteus-passenger.webp",
        "Painted three-quarter view of Proteus Passenger, an ivory, vermilion, "
        "and blue fifty-ton modular shuttle with six paired window groups.",
        26.0,
    ),
    22: (
        "assets/ships/family-022-albatross.webp",
        "Painted three-quarter view of Albatross, an ivory, vermilion, and blue "
        "forty-ton pinnace with eight passenger-window bays and a freight door.",
        23.5,
    ),
    151: (
        "assets/ships/family-022-albatross.webp",
        "Painted three-quarter view of Albatross, an ivory, vermilion, and blue "
        "forty-ton pinnace with eight passenger-window bays and a freight door.",
        23.5,
    ),
    24: (
        "assets/ships/ship-024-proteus-prospector.webp",
        "Painted three-quarter view of Proteus Prospector, an avocado, ochre, "
        "and brick-red fifty-ton mining boat with workshop and drone shutters.",
        26.0,
    ),
    26: (
        "assets/ships/ship-026-pym.webp",
        "Painted three-quarter view of Pym, a yellow and cobalt hundred-ton "
        "dispatch courier with a mixed triple turret and broad freight door.",
        31.0,
    ),
    27: (
        "assets/ships/family-027-mercator.webp",
        "Painted three-quarter view of Mercator, an ivory, blue, and vermilion "
        "hundred-ton light trader with a broad freight door and empty hardpoint.",
        32.5,
    ),
    28: (
        "assets/ships/family-027-mercator.webp",
        "Painted three-quarter view of Mercator, an ivory, blue, and vermilion "
        "hundred-ton light trader with a broad freight door and empty hardpoint.",
        32.5,
    ),
    30: (
        "assets/ships/family-030-goliath.webp",
        "Painted three-quarter view of Goliath, a navy, white, and red ninety-ton "
        "assault lander with two airlocks, a stores ramp, and beam-laser turret.",
        30.0,
    ),
    31: (
        "assets/ships/family-027-mercator.webp",
        "Painted three-quarter view of Mercator, an ivory, blue, and vermilion "
        "hundred-ton light trader with a broad freight door and empty hardpoint.",
        32.5,
    ),
    32: (
        "assets/ships/family-030-goliath.webp",
        "Painted three-quarter view of Goliath, a navy, white, and red ninety-ton "
        "assault lander with two airlocks, a stores ramp, and beam-laser turret.",
        30.0,
    ),
    33: (
        "assets/ships/ship-033-ligeia.webp",
        "Painted three-quarter view of Ligeia, an aubergine hundred-ton covert "
        "courier with split cargo and hangar shutters and point defense.",
        31.0,
    ),
    134: (
        "assets/ships/family-018-wayfarer-armed.webp",
        "Painted three-quarter view of Wayfarer Armed, an emerald and cream "
        "thirty-ton fast boat with a trainable dorsal beam-laser turret.",
        18.8,
    ),
    165: (
        "assets/ships/family-018-wayfarer-armed.webp",
        "Painted three-quarter view of Wayfarer Armed, an emerald and cream "
        "thirty-ton fast boat with a trainable dorsal beam-laser turret.",
        18.8,
    ),
    158: (
        "assets/ships/family-005-charon.webp",
        "Painted three-quarter view of Charon, an ivory, vermilion, and blue "
        "ten-ton streamlined armored passenger launch with three cabin windows.",
        12.5,
    ),
    159: (
        "assets/ships/ship-159-proteus-mercy.webp",
        "Painted three-quarter view of Proteus Mercy, a white, cyan, and lime "
        "fifty-ton medical cutter with two airlocks and ward windows.",
        26.0,
    ),
    145: (
        "assets/ships/family-007-caduceus-concord.webp",
        "Painted three-quarter view of Caduceus, an ivory and blue twenty-ton "
        "armored fast launch with a closed dorsal hardpoint.",
        17.5,
    ),
    146: (
        "assets/ships/family-007-caduceus-concord.webp",
        "Painted three-quarter view of Caduceus, an ivory and blue twenty-ton "
        "armored fast launch with a closed dorsal hardpoint.",
        17.5,
    ),
    176: (
        "assets/ships/ship-176-boreas.webp",
        "Painted three-quarter view of Boreas, an ivory and blue twenty-ton "
        "armored captain's gig with three paired cabin-window groups.",
        18.5,
    ),
    177: (
        "assets/ships/ship-177-zephyrus.webp",
        "Painted three-quarter view of Zephyrus, an ivory, vermilion, and blue "
        "twenty-ton armored flag barge with four private-cabin windows.",
        18.5,
    ),
    178: (
        "assets/ships/ship-178-castor.webp",
        "Painted three-quarter view of Castor, a navy and ivory thirty-ton "
        "armored personnel boat with four cabin windows and a beam-laser turret.",
        21.0,
    ),
    179: (
        "assets/ships/ship-179-pollux.webp",
        "Painted three-quarter view of Pollux, a navy and ivory thirty-ton "
        "armored marine boat with six shuttered troop-compartment bays.",
        21.0,
    ),
    181: (
        "assets/ships/family-018-wayfarer-cargo.webp",
        "Painted three-quarter view of Wayfarer Cargo, an ivory, vermilion, "
        "and blue thirty-ton unarmed utility boat with seven cabin windows.",
        18.8,
    ),
    187: (
        "assets/ships/family-018-wayfarer-cargo.webp",
        "Painted three-quarter view of Wayfarer Cargo, an ivory, vermilion, "
        "and blue thirty-ton unarmed utility boat with seven cabin windows.",
        18.8,
    ),
    188: (
        "assets/ships/ship-188-proteus-surveyor.webp",
        "Painted three-quarter view of Proteus Surveyor, a white, cyan, and "
        "lime fifty-ton survey cutter with instruments and an air-raft door.",
        26.0,
    ),
    185: (
        "assets/ships/family-007-caduceus-venture.webp",
        "Painted three-quarter view of Caduceus, a sunflower and cobalt "
        "twenty-ton armored fast launch with a closed dorsal hardpoint.",
        17.5,
    ),
    189: (
        "assets/ships/family-007-caduceus-venture.webp",
        "Painted three-quarter view of Caduceus, a sunflower and cobalt "
        "twenty-ton armored fast launch with a closed dorsal hardpoint.",
        17.5,
    ),
    209: (
        "assets/ships/ship-209-proteus-modular.webp",
        "Painted three-quarter view of Proteus Modular, an ivory, vermilion, "
        "and blue fifty-ton unfaired chassis with a sealed central module.",
        25.5,
    ),
    212: (
        "assets/ships/ship-212-wayfarer-boarding.webp",
        "Painted three-quarter view of Wayfarer Boarding, an emerald and cream "
        "thirty-ton boarding boat with a fixed beam emitter and airlock rails.",
        18.8,
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
    overrides = {
        "captains-gig": "Captain's Gig",
        "ship's-boat": "Ship's Boat",
    }
    if value in overrides:
        return overrides[value]
    return value.replace("-", " ").title()


def format_tons(millitons: int) -> str:
    tons = millitons / 1000
    value = f"{tons:,.3f}".rstrip("0").rstrip(".")
    return f"{value} ton{'s' if tons != 1 else ''}"


def parameterized_equipment_name(item: dict[str, object]) -> str:
    details = []
    if "beds" in item:
        beds = int(item["beds"])
        details.append(f"{beds} bed{'s' if beds != 1 else ''}")
    if "contained_millitons" in item:
        details.append(f"{format_tons(int(item['contained_millitons']))} contained")
    for key, value in item.items():
        if key not in {"id", "quantity", "beds", "contained_millitons"}:
            details.append(f"{display_term(key)}: {value}")
    name = display_term(str(item["id"]))
    return f"{name} ({', '.join(details)})" if details else name


def catalog_records() -> list[dict[str, object]]:
    names = tomllib.loads((SHIP_CATALOG / "names.toml").read_text(encoding="utf-8"))
    family_names = {
        entry["family_id"]: entry["display_name"] for entry in names["family_name"]
    }
    path_names = {entry["path_id"]: entry for entry in names["path_name"]}
    shipbuilding = tomllib.loads(SHIPBUILDING_CORE.read_text(encoding="utf-8"))
    armor_points_per_layer = {
        entry["id"]: entry["protection_per_layer"]
        for entry in shipbuilding["armor"]
    }
    records: list[dict[str, object]] = []
    for catalog_id, (art_path, art_alt, length_m) in PUBLISHED_SHIP_ART.items():
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
        armor = data.get("armor")
        armor_layers = armor.get("layers", 0) if armor else 0
        if armor and "points" in armor:
            armor_points = armor["points"]
        elif armor:
            try:
                armor_points = armor_layers * armor_points_per_layer[armor["id"]]
            except KeyError as error:
                raise RuntimeError(
                    f"unknown layered armor ID in {source}: {armor['id']}"
                ) from error
        else:
            armor_points = 0
        hull_options = []
        for item in data.get("hull_options", []):
            quantity = int(item.get("quantity", 1))
            option_name = display_term(item["id"])
            hull_options.append(
                option_name if quantity == 1 else f"{quantity} × {option_name}"
            )
        equipment = []
        equipment_entries = []
        for item in data.get("equipment", []):
            quantity = item.get("quantity", 1)
            equipment_name = display_term(item["id"])
            if item["id"] == "acceleration-seat":
                equipment.append(f"{quantity} passenger seats")
            elif quantity == 1:
                equipment.append(equipment_name)
            else:
                equipment.append(f"{quantity} × {equipment_name}")
            equipment_entries.append(
                {
                    "name": f"{equipment_name}{'s' if quantity != 1 else ''}",
                    "quantity": quantity,
                }
            )
        for item in data.get("parameterized_equipment", []):
            quantity = int(item.get("quantity", 1))
            equipment_name = parameterized_equipment_name(item)
            equipment.append(
                equipment_name if quantity == 1 else f"{quantity} × {equipment_name}"
            )
            equipment_entries.append(
                {"name": equipment_name, "quantity": quantity}
            )
        for item in data.get("hangars", []):
            quantity = int(item.get("quantity", 1))
            hangar_name = (
                f"{display_term(item['id'])} "
                f"({format_tons(item['contained_millitons'])} contained)"
            )
            equipment.append(
                hangar_name if quantity == 1 else f"{quantity} × {hangar_name}"
            )
            equipment_entries.append(
                {"name": hangar_name, "quantity": quantity}
            )
        airlocks = data.get("airlocks")
        if airlocks:
            equipment.append(
                "Pressure airlock" if airlocks == 1 else f"{airlocks} pressure airlocks"
            )
        mount_entries = []
        for item in data.get("mounts", []):
            mount_name = display_term(item["id"])
            if item.get("fixed", False):
                mount_name = f"Fixed {mount_name}"
            if item.get("pop_up", False):
                mount_name = f"Pop-Up {mount_name}"
            mount_entries.append(
                {
                    "name": mount_name,
                    "weapons": [display_term(weapon) for weapon in item["weapons"]],
                    "quantity": int(item.get("quantity", 1)),
                }
            )
        for item in data.get("point_defense", []):
            mount_entries.append(
                {
                    "name": display_term(item["mount_id"]),
                    "weapons": [display_term(item["weapon_id"])],
                    "quantity": int(item.get("quantity", 1)),
                }
            )
        armament_parts = []
        for entry in mount_entries:
            mount_name = entry["name"]
            if entry["quantity"] != 1:
                mount_name = f"{entry['quantity']} × {mount_name}"
            armament_parts.append(
                f"{mount_name}: {joined_terms(entry['weapons'])}"
            )
        armament = " · ".join(armament_parts) or "None installed"
        software = []
        for item in data.get("software", []):
            software_name = display_term(item["id"])
            if "level" in item:
                software_name = f"{software_name}/{item['level']}"
            software.append(software_name)
        ammunition_entries = []
        ammunition = []
        for item in data.get("ammunition", []):
            quantity = item["quantity"]
            ammunition_name = display_term(item["id"])
            plural_name = f"{ammunition_name}{'s' if quantity != 1 else ''}"
            ammunition.append(f"{quantity} × {plural_name}")
            ammunition_entries.append(
                {"name": plural_name, "quantity": quantity}
            )
        if not equipment and not mount_entries and not ammunition_entries:
            equipment.append("Priority cargo module")
        crew_entries = [
            {"role": display_term(item["role"]), "quantity": item["quantity"]}
            for item in data.get("crew", [])
        ]
        crew = sum(item["quantity"] for item in data.get("crew", []))
        control = data.get("control")
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
                "page": f"ship-{catalog_id:03}-{slug(catalog['display_name'])}.html",
                "design_id": data["design_id"],
                "schema_version": data["schema_version"],
                "revision": data["revision"],
                "ruleset": display_term(data["ruleset_id"].replace(".", " ")),
                "source_ids": data["source_ids"],
                "tech_level": data["tech_level"],
                "standard_design": data["standard_design"],
                "armor_id": display_term(armor["id"]) if armor else "None",
                "armor_layers": armor_layers,
                "armor_points": armor_points,
                "electronics": display_term(data["electronics"]),
                "status": display_term(catalog["status"]),
                "progression_stage": display_term(catalog["progression_stage"]),
                "vessel_kind": display_term(catalog["vessel_kind"]),
                "secondary_roles": [display_term(value) for value in catalog["secondary_roles"]],
                "mission_tags": [display_term(value) for value in catalog["mission_tags"]],
                "ogc_designations": catalog["open_game_content_designations"],
                "tons": int(hull_match.group(1)),
                "hull_id": data["hull"]["id"],
                "configuration": display_term(data["hull"]["configuration"]),
                "hull_options": hull_options,
                "maneuver_drive": data["drives"]["maneuver"],
                "power_plant": data["drives"]["power"],
                "jump_drive": data["drives"].get("jump"),
                "jump_distance": data["fuel"].get("jump_distance"),
                "jump_count": data["fuel"].get("jump_count"),
                "control": (
                    display_term(control["id"]) if control else "Standard bridge"
                ),
                "additional_passengers": (
                    control["additional_passengers"]
                    if control else "Not applicable"
                ),
                "bridge_options": [
                    display_term(value) for value in data.get("bridge_options", [])
                ],
                "computer": display_term(data["computer"]["id"]),
                "computer_options": [display_term(value) for value in data["computer"]["options"]],
                "software": software,
                "additional_fire_control_stations": data.get(
                    "additional_fire_control_stations", 0
                ),
                "unused_fire_control_stations": data.get(
                    "unused_fire_control_stations", 0
                ),
                "airlocks": airlocks,
                "crew": crew,
                "crew_entries": crew_entries,
                "endurance": data["fuel"]["power_plant_weeks"],
                "cargo": format_tons(data.get("cargo_millitons", 0)),
                "equipment": " · ".join(equipment),
                "equipment_entries": equipment_entries,
                "armament": armament,
                "mount_entries": mount_entries,
                "ammunition": " · ".join(ammunition) or "None carried",
                "ammunition_entries": ammunition_entries,
                "assertions": data.get("assertions", {}),
                "raw_data": data,
                "art_path": art_path,
                "art_alt": art_alt,
                "length_m": length_m,
            }
        )
    return sorted(records, key=lambda ship: (ship["family_id"], ship["catalog_id"]))


def ship_catalog_page(records: list[dict[str, object]] | None = None) -> str:
    if records is None:
        records = catalog_records()
    index_links = []
    for ship in records:
        search_text = " ".join(
            str(value)
            for value in (
                ship["name"],
                ship["role"],
                ship["family_name"],
                ship["path_name"],
                ship["yard_name"],
                ship["configuration"],
                *ship["hull_options"],
                ship["equipment"],
                ship["armament"],
                ship["ammunition"],
                *ship["bridge_options"],
                *ship["software"],
                *ship["secondary_roles"],
                *ship["mission_tags"],
                *ship["description"],
            )
        ).lower()
        index_links.append(
            f'<a href="{ship["page"]}" data-catalog-entry '
            f'data-family="{ship["family_id"]}" data-path="{ship["path_id"]}" '
            f'data-search="{html.escape(search_text, quote=True)}">'
            f'<span>{ship["catalog_id"]:03}</span><strong>{html.escape(ship["name"])}</strong>'
            f'<small>{html.escape(ship["family_name"])} family · {ship["tons"]} tons · '
            f'{html.escape(ship["role"])}<br>{html.escape(ship["path_name"])}</small>'
            f'<b>Open full dossier →</b></a>'
        )
    families = sorted({(ship["family_id"], ship["family_name"]) for ship in records})
    paths = sorted({(ship["path_id"], ship["path_name"]) for ship in records})
    family_options = "".join(
        f'<option value="{family_id}">{html.escape(family_name)}</option>'
        for family_id, family_name in families
    )
    path_options = "".join(
        f'<option value="{path_id}">{html.escape(path_name)}</option>'
        for path_id, path_name in paths
    )
    body = f"""
<header class="document-hero catalog-hero">
  <p class="eyebrow">Office of Vessel Recognition // SC-01</p>
  <h1>Ship Catalog</h1>
  <p class="catalog-intro">Issued to captains, brokers, port officials, and boarding crews. Match silhouette and visible fittings before trusting a transponder return: local refits, improvised repairs, and false registry marks are common.</p>
  <dl class="catalog-overview" aria-label="Catalog publication status">
    <div><dt>Dossiers issued</dt><dd>{len(records):02}</dd></div>
    <div><dt>Distinct plates</dt><dd>{len({ship['art_path'] for ship in records}):02}</dd></div>
    <div><dt>Complete families</dt><dd>{len(families):02}</dd></div>
    <div><dt>Catalog designs</dt><dd>213</dd></div>
  </dl>
</header>
<div class="catalog-registry">
  <div class="catalog-controls" role="search" aria-label="Search issued ship plates">
    <label class="catalog-query" for="ship-query"><span>Search registry</span><input id="ship-query" type="search" autocomplete="off" placeholder="Name, role, yard, or visible fit…"></label>
    <label for="ship-family"><span>Family</span><select id="ship-family"><option value="">All issued families</option>{family_options}</select></label>
    <label for="ship-path"><span>Shipyard path</span><select id="ship-path"><option value="">All issued paths</option>{path_options}</select></label>
    <p id="ship-results" role="status" aria-live="polite">Showing all {len(records)} issued entries.</p>
  </div>
  <section class="catalog-directory" aria-labelledby="catalog-index-title">
    <header><p class="section-index">Registry finder</p><h2 id="catalog-index-title">Issued dossiers</h2><p>This register stays compact for rapid filtering. Open a vessel to see its complete recognition plate, operational profile, construction record, crew, equipment, and provenance.</p></header>
    <nav class="catalog-index" id="catalog-index" aria-label="Issued ship plates">{''.join(index_links)}</nav>
    <div id="no-ship-results" class="no-results" hidden><strong>No issued plate matches.</strong><p>Clear a filter or try a broader registry term.</p></div>
    <p class="catalog-source"><span>Registry note</span> Only vessels with an approved family plate appear here. The active construction catalog currently contains 213 designs.</p>
  </section>
</div>
"""
    return page_shell(
        "Ship Catalog",
        "Illustrated player-facing ship catalog for Cepheus Trader, with complete recognition and construction dossiers.",
        "catalog",
        body,
    )


def record_list(rows: list[tuple[str, object]]) -> str:
    return '<dl class="ship-record-list">' + "".join(
        f"<div><dt>{html.escape(label)}</dt><dd>{html.escape(str(value))}</dd></div>"
        for label, value in rows
    ) + "</dl>"


def joined_terms(values: list[str]) -> str:
    return " · ".join(values) if values else "None"


def assertion_value(key: str, value: object) -> str:
    if key.endswith("_millitons") and isinstance(value, int):
        return f"{value:,} millitons / {format_tons(value)}"
    if key.endswith("_credits") and isinstance(value, int):
        return f"{value:,} Cr"
    if key == "thrust_g":
        return f"{value} g"
    if key == "construction_weeks":
        return f"{value} weeks"
    return str(value)


def source_ledger(value: object) -> str:
    if isinstance(value, dict):
        if not value:
            return '<span class="ledger-empty">None recorded</span>'
        rows = []
        for key, child in value.items():
            label = key.replace("_", " ").title()
            rows.append(
                f"<div><dt>{html.escape(label)}</dt><dd>{source_ledger(child)}</dd></div>"
            )
        return f'<dl class="source-ledger">{"".join(rows)}</dl>'
    if isinstance(value, list):
        if not value:
            return '<span class="ledger-empty">None recorded</span>'
        list_class = (
            "ledger-records"
            if any(isinstance(item, dict) for item in value)
            else "ledger-values"
        )
        return f'<ol class="{list_class}">' + "".join(
            f"<li>{source_ledger(item)}</li>" for item in value
        ) + "</ol>"
    if isinstance(value, bool):
        return "Yes" if value else "No"
    return html.escape(str(value))


def ship_detail_page(ship: dict[str, object], records: list[dict[str, object]]) -> str:
    paragraphs = "".join(f"<p>{html.escape(text)}</p>" for text in ship["description"])
    weeks = ship["endurance"]
    family_members = [item for item in records if item["family_id"] == ship["family_id"]]
    member_links = "".join(
        f'<a href="{item["page"]}"{f" aria-current=\"page\"" if item is ship else ""}>'
        f'<span>{item["catalog_id"]:03}</span><strong>{html.escape(item["name"])}</strong>'
        f'<small>{html.escape(item["role"])}</small></a>'
        for item in family_members
    )
    position = family_members.index(ship)
    previous_link = (
        f'<a rel="prev" href="{family_members[position - 1]["page"]}">← '
        f'{html.escape(family_members[position - 1]["name"])}</a>'
        if position > 0 else "<span></span>"
    )
    next_link = (
        f'<a rel="next" href="{family_members[position + 1]["page"]}">'
        f'{html.escape(family_members[position + 1]["name"])} →</a>'
        if position + 1 < len(family_members) else "<span></span>"
    )
    crew = "".join(
        f'<li><strong>{entry["quantity"]}</strong> {html.escape(entry["role"])}'
        f'{"s" if entry["quantity"] != 1 else ""}</li>'
        for entry in ship["crew_entries"]
    ) or "<li>None recorded</li>"
    equipment = "".join(
        f'<li><strong>{entry["quantity"]}</strong> {html.escape(entry["name"])}</li>'
        for entry in ship["equipment_entries"]
    )
    mount_items = []
    for entry in ship["mount_entries"]:
        mount_name = html.escape(entry["name"])
        if entry["quantity"] != 1:
            mount_name = f'{entry["quantity"]} × {mount_name}'
        mount_items.append(
            f'<li><strong>{mount_name}</strong> '
            f'{html.escape(joined_terms(entry["weapons"]))}</li>'
        )
    mounts = "".join(mount_items)
    ammunition = "".join(
        f'<li><strong>{entry["quantity"]}</strong> {html.escape(entry["name"])}</li>'
        for entry in ship["ammunition_entries"]
    )
    installed_fit = equipment + mounts + ammunition
    if not installed_fit:
        installed_fit = "<li>No separate equipment, armament, or ammunition recorded</li>"
    recognized_fit_parts = (
        [joined_terms(ship["hull_options"])] if ship["hull_options"] else []
    )
    if ship["equipment"]:
        recognized_fit_parts.append(ship["equipment"])
    if ship["armament"] != "None installed":
        recognized_fit_parts.append(ship["armament"])
    if ship["ammunition"] != "None carried":
        recognized_fit_parts.append(ship["ammunition"])
    recognized_fit = " · ".join(recognized_fit_parts) or "No installed fit recorded"
    source_ids = "".join(f"<li><code>{html.escape(value)}</code></li>" for value in ship["source_ids"])
    ogc = "".join(f"<p>{html.escape(value)}</p>" for value in ship["ogc_designations"])
    assertions = ""
    if ship["assertions"]:
        assertion_rows = [
            (key.replace("_", " ").title(), assertion_value(key, value))
            for key, value in ship["assertions"].items()
        ]
        assertions = f"""
      <section class="ship-assertions">
        <h3>Verified performance and procurement</h3>
        <p>Construction assertions carried by this source record.</p>
        {record_list(assertion_rows)}
      </section>"""
    body = f"""
<article class="ship-detail path-{ship['path_id']}">
  <header class="ship-detail-hero">
    <nav class="ship-breadcrumb" aria-label="Breadcrumb"><a href="ships.html">Ship Catalog</a><span>/</span><span>Family {ship['family_id']:03}</span><span>/</span><span>Entry {ship['catalog_id']:03}</span></nav>
    <div class="ship-detail-heading">
      <div><p class="eyebrow">Office of Vessel Recognition // {ship['tag']}</p><h1>{html.escape(ship['name'])}</h1><p>{html.escape(ship['role'])}</p></div>
      <div class="ship-detail-yard"><span>Path {ship['path_id']:02} / {html.escape(ship['path_name'])}</span><strong>{html.escape(ship['yard_name'])}</strong></div>
    </div>
  </header>
  <div class="ship-detail-main">
    <figure class="ship-detail-plate">
      <img src="{ship['art_path']}" alt="{html.escape(ship['art_alt'], quote=True)}" width="1536" height="1024">
      <figcaption><span>{ship['tag']} / Canonical recognition plate</span><span class="ship-scale">{ship['length_m']:g} m overall</span></figcaption>
    </figure>
    <dl class="ship-detail-summary">
      <div><dt>Displacement</dt><dd>{ship['tons']} tons</dd></div>
      <div><dt>Configuration</dt><dd>{ship['configuration']}</dd></div>
      <div><dt>Tech level</dt><dd>{ship['tech_level']}</dd></div>
      <div><dt>Crew</dt><dd>{ship['crew']}</dd></div>
      <div><dt>Endurance</dt><dd>{weeks} week{'s' if weeks != 1 else ''}</dd></div>
      <div><dt>Cargo</dt><dd>{ship['cargo']}</dd></div>
    </dl>
    <div class="ship-detail-profile">
      <section><p class="section-index">Operational profile</p><h2>Recognition and use</h2><div class="ship-description">{paragraphs}</div></section>
      <aside><p class="ship-fit"><span>Recognized fit</span>{html.escape(recognized_fit)}</p><p class="ship-yard">Yard pattern: <strong>{html.escape(ship['yard_name'])}</strong>.</p><div class="record-tags" aria-label="Mission tags">{''.join(f'<span>{html.escape(value)}</span>' for value in ship['mission_tags'])}</div></aside>
    </div>
    <section class="ship-construction" aria-labelledby="construction-title">
      <header><p class="section-index">Complete record / Revision {ship['revision']}</p><h2 id="construction-title">Construction dossier</h2><p>All fields below are read from the active catalog record. “None” is explicit where the record contains no fitted item or option.</p></header>
      <div class="ship-record-grid">
        <section><h3>Registry</h3>{record_list([
            ('Catalog entry', f'{ship["catalog_id"]:03}'),
            ('Design ID', ship['design_id']),
            ('Schema version', ship['schema_version']),
            ('Record revision', ship['revision']),
            ('Status', ship['status']),
            ('Vessel kind', ship['vessel_kind']),
            ('Progression stage', ship['progression_stage']),
            ('Family', f'{ship["family_id"]:03} / {ship["family_name"]}'),
            ('Shipyard path', f'{ship["path_id"]:02} / {ship["path_name"]}'),
            ('Standard design', 'Yes' if ship['standard_design'] else 'No'),
        ])}</section>
        <section><h3>Hull and systems</h3>{record_list([
            ('Ruleset', ship['ruleset']),
            ('Tech level', ship['tech_level']),
            ('Hull code', ship['hull_id']),
            ('Displacement', f'{ship["tons"]} tons'),
            ('Configuration', ship['configuration']),
            ('Hull options', joined_terms(ship['hull_options'])),
            ('Armor', ship['armor_id']),
            ('Armor layers', ship['armor_layers'] if ship['armor_layers'] else ('Not layer-rated' if ship['armor_points'] else 'None')),
            ('Armor points', ship['armor_points']),
            ('Electronics', ship['electronics']),
            ('Airlocks', ship['airlocks'] if ship['airlocks'] is not None else 'Not separately recorded'),
            ('Cargo', ship['cargo']),
            ('Armament', ship['armament']),
            ('Ammunition', ship['ammunition']),
        ])}</section>
        <section><h3>Drives and control</h3>{record_list([
            ('Maneuver drive', ship['maneuver_drive']),
            ('Power plant', ship['power_plant']),
            ('Jump drive', ship['jump_drive'] or 'None installed'),
            ('Jump distance', ship['jump_distance'] if ship['jump_distance'] is not None else 'Not applicable'),
            ('Jumps carried', ship['jump_count'] if ship['jump_count'] is not None else 'Not applicable'),
            ('Power endurance', f'{weeks} week{"s" if weeks != 1 else ""}'),
            ('Control', ship['control']),
            ('Additional passengers', ship['additional_passengers']),
            ('Bridge options', joined_terms(ship['bridge_options'])),
            ('Computer', ship['computer']),
            ('Computer options', joined_terms(ship['computer_options'])),
            ('Software', joined_terms(ship['software'])),
            ('Additional fire-control stations', ship['additional_fire_control_stations']),
            ('Unused fire-control stations', ship['unused_fire_control_stations']),
        ])}</section>
      </div>
      <div class="ship-manifest-grid">
        <section><h3>Crew complement</h3><ul>{crew}</ul></section>
        <section><h3>Installed equipment, armament, and ammunition</h3><ul>{installed_fit}</ul></section>
        <section><h3>Mission classification</h3>{record_list([
            ('Primary role', ship['role']),
            ('Secondary roles', joined_terms(ship['secondary_roles'])),
            ('Mission tags', joined_terms(ship['mission_tags'])),
        ])}</section>
      </div>
      {assertions}
      <details class="complete-ledger">
        <summary><span>Complete source ledger</span><small>Every field in the catalog record, including empty lists and construction assertions.</small></summary>
        <div class="complete-ledger-body">{source_ledger(ship['raw_data'])}</div>
      </details>
    </section>
    <section class="ship-provenance"><div><p class="section-index">Record provenance</p><h2>Sources and designation</h2><p>Source record: <code>catalog/ships/ship-{ship['catalog_id']}.toml</code></p>{ogc}</div><ul>{source_ids}</ul></section>
    <nav class="family-navigation" aria-label="{html.escape(ship['family_name'], quote=True)} family vessels"><header><p class="section-index">Family {ship['family_id']:03}</p><h2>{html.escape(ship['family_name'])} family</h2></header><div>{member_links}</div></nav>
    <nav class="dossier-pagination" aria-label="Previous and next family vessels">{previous_link}<a href="ships.html">All issued dossiers</a>{next_link}</nav>
  </div>
</article>
"""
    return page_shell(
        ship["name"],
        f"Complete recognition and construction dossier for the {ship['name']} {ship['role'].lower()}.",
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
    ships = catalog_records()
    pages = {
        "index.html": landing_page(),
        "ships.html": ship_catalog_page(ships),
        "reference.html": reference_page(),
        "beginner-help.html": beginner_help_page(topics),
    }
    pages.update({ship["page"]: ship_detail_page(ship, ships) for ship in ships})
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
