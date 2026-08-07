#!/usr/bin/env python3
"""Generate a pure-GFM edition of the Cepheus Engine SRD PDF.

The PDF uses a mixture of one-column prose, two-column prose, and geometric
tables.  Poppler's TSV output supplies word coordinates; this script uses
those coordinates to restore reading order and turn the geometric tables
into GitHub-Flavored Markdown pipe tables.
"""

from __future__ import annotations

import csv
import io
import re
import subprocess
import sys
import tempfile
import unicodedata
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

import numpy as np
from PIL import Image
from scipy import ndimage


ROOT = Path(__file__).resolve().parent.parent
PDF = ROOT / "cepodnew.pdf"
OUT = Path(__file__).resolve().parent
PAGE_MID = 306.0
PAGE_LEFT = 36.0
PAGE_RIGHT = 576.0
# Body copy occasionally reaches y=752; the repeating footer begins at y=769.
FOOTER_TOP = 765.0

CAREER_GROUPS = {
    20: ["Athlete", "Aerospace Defense", "Agent", "Barbarian", "Belter", "Bureaucrat"],
    21: ["Athlete", "Aerospace Defense", "Agent", "Barbarian", "Belter", "Bureaucrat"],
    22: ["Colonist", "Diplomat", "Drifter", "Entertainer", "Hunter", "Marine"],
    23: ["Colonist", "Diplomat", "Drifter", "Entertainer", "Hunter", "Marine"],
    24: ["Maritime Defense", "Mercenary", "Merchant", "Navy", "Noble", "Physician"],
    25: ["Maritime Defense", "Mercenary", "Merchant", "Navy", "Noble", "Physician"],
    26: ["Pirate", "Rogue", "Scientist", "Scout", "Surface Defense", "Technician"],
    27: ["Pirate", "Rogue", "Scientist", "Scout", "Surface Defense", "Technician"],
}


@dataclass(frozen=True)
class Document:
    filename: str
    title: str
    printed_start: int
    printed_end: int
    book: str | None
    chapter: int | None

    @property
    def physical_start(self) -> int:
        return self.printed_start + 1

    @property
    def physical_end(self) -> int:
        return self.printed_end + 1


DOCUMENTS = [
    Document("00-introduction.md", "Introduction", 3, 10, None, None),
    Document("01-character-creation.md", "Chapter 1: Character Creation", 11, 31, "Book One: Characters", 1),
    Document("02-skills.md", "Chapter 2: Skills", 32, 38, "Book One: Characters", 2),
    Document("03-psionics.md", "Chapter 3: Psionics", 39, 44, "Book One: Characters", 3),
    Document("04-equipment.md", "Chapter 4: Equipment", 45, 62, "Book One: Characters", 4),
    Document("05-personal-combat.md", "Chapter 5: Personal Combat", 63, 72, "Book One: Characters", 5),
    Document("06-off-world-travel.md", "Chapter 6: Off-World Travel", 73, 81, "Book Two: Starships and Interstellar Travel", 6),
    Document("07-trade-and-commerce.md", "Chapter 7: Trade and Commerce", 82, 84, "Book Two: Starships and Interstellar Travel", 7),
    Document("08-ship-design-and-construction.md", "Chapter 8: Ship Design and Construction", 85, 100, "Book Two: Starships and Interstellar Travel", 8),
    Document("09-common-vessels.md", "Chapter 9: Common Vessels", 101, 107, "Book Two: Starships and Interstellar Travel", 9),
    Document("10-space-combat.md", "Chapter 10: Space Combat", 108, 118, "Book Two: Starships and Interstellar Travel", 10),
    Document("11-environments-and-hazards.md", "Chapter 11: Environments and Hazards", 119, 121, "Book Three: Referees", 11),
    Document("12-worlds.md", "Chapter 12: Worlds", 122, 129, "Book Three: Referees", 12),
    Document("13-planetary-wilderness-encounters.md", "Chapter 13: Planetary Wilderness Encounters", 130, 137, "Book Three: Referees", 13),
    Document("14-social-encounters.md", "Chapter 14: Social Encounters", 138, 142, "Book Three: Referees", 14),
    Document("15-starship-encounters.md", "Chapter 15: Starship Encounters", 143, 145, "Book Three: Referees", 15),
    Document("16-refereeing-the-game.md", "Chapter 16: Refereeing the Game", 146, 149, "Book Three: Referees", 16),
    Document("17-adventures.md", "Chapter 17: Adventures", 150, 152, "Book Three: Referees", 17),
    Document("legal.md", "Legal", 153, 154, None, None),
]

CHAPTER_BY_NUMBER = {doc.chapter: doc for doc in DOCUMENTS if doc.chapter}
PAGE_TO_DOCUMENT = {
    page: doc
    for doc in DOCUMENTS
    for page in range(doc.printed_start, doc.printed_end + 1)
}

HEADER_OVERRIDES = {
    "Table: Characteristic Modifier by Score Range": [
        "Score Range",
        "PseudoHex",
        "Characteristic Modifier",
    ],
    "Table: Medical Bills": [
        "Career",
        "Roll of 4+",
        "Roll of 8+",
        "Roll of 12+",
    ],
    "Table: Available Skills": [
        "Basic Skills",
        "Weapon Skills",
        "Transport Skills",
    ],
    "Table: Ship Armor by Type": ["Armor Type", "TL", "Protection", "Cost"],
    "Table: Clairvoyance": ["Ability", "Difficulty", "Timing", "Cost"],
    "Table: Communications Equipment": [
        "Communicator",
        "TL",
        "Cost",
        "Wgt",
        "Range",
    ],
    "Table: Explosives": ["Weapon", "TL", "Damage", "Radius", "Cost (Cr)"],
    "Table: Personal Devices": ["Description", "TL", "Cost", "Wgt"],
    "Table: Sensory Aids": ["Description", "TL", "Cost", "Wgt"],
    "Table: Shelters": ["Description", "TL", "Cost", "Wgt"],
    "Table: Survival Equipment": ["Description", "TL", "Cost", "Wgt"],
    "Table: Tools": ["Description", "TL", "Cost", "Wgt"],
    "Table: Common Vehicles": [
        "Vehicle",
        "TL",
        "Skill",
        "Agi",
        "Spd",
        "C&P",
        "O/C",
        "Armor",
        "Hull",
        "Struc",
        "Wpns",
        "Cost (KCr)",
    ],
    "Table: Common Heavy Weapons": [
        "Weapon",
        "TL",
        "Cost",
        "Wgt",
        "RoF",
        "Range",
        "Damage",
        "Recoil",
        "LL",
    ],
    "Table: Common Heavy Weapon Ammunition": [
        "Weapon",
        "TL",
        "Cost",
        "Wgt",
        "Rounds",
    ],
    "Table: Attack Difficulties by Weapon Type": [
        "Weapon",
        "Personal",
        "Close",
        "Short",
        "Medium",
        "Long",
        "Very Long",
        "Distant",
        "Very Distant",
        "Extreme",
        "Continental",
    ],
    "Table: System Degradation": ["Roll", "Number of Hits"],
    "Table: Potential Law Enforcement Encounters": [
        "Situation",
        "DM",
        "Response",
    ],
    "Table: Drive Performance by Hull Volume, Smaller Hulls": [
        "Drive Code",
        "100",
        "200",
        "300",
        "400",
        "500",
        "600",
        "700",
        "800",
        "900",
        "1000",
    ],
    "Table: Drive Performance by Hull Volume, Larger Hulls": [
        "Drive Code",
        "1200",
        "1400",
        "1600",
        "1800",
        "2000",
        "3000",
        "4000",
        "5000",
    ],
    "Table: Ship Computer Models": ["Computer", "TL", "Rating", "Cost"],
    "Table: Turret Displacement and Cost": [
        "Weapon",
        "TL",
        "Tons",
        "Cost (MCr)",
    ],
    "Table: Turret Weapons": [
        "Weapon",
        "TL",
        "Optimum Range",
        "Damage",
        "Cost (MCr)",
    ],
    "Table: Bay Weapons": ["Weapon", "TL", "Range", "Damage", "Cost (MCr)"],
    "Table: Hyperspace Portal Size": ["Rating", "Size", "Rating", "Size"],
    "Table: Ship Hull by Displacement": [
        "Hull",
        "Hull Code",
        "Price (MCr)",
        "Construction Time (weeks)",
    ],
    "Table: Small Craft Drive Performance by Hull Volume": [
        "Drive Code",
        "10",
        "15",
        "20",
        "25",
        "30",
        "35",
        "40",
        "45",
        "50",
        "55",
        "60",
        "65",
        "70",
        "75",
        "80",
        "85",
        "90",
        "95",
    ],
    "Table: Space Combat Attack Difficulties by Weapon Type": [
        "Weapon",
        "Adjacent",
        "Close",
        "Short",
        "Medium",
        "Long",
        "Very Long",
        "Distant",
    ],
    "Table: Missile Launch Range": ["Range", "Turns to Impact"],
    "Table: Missile To-Hit By Skill Check Effect": [
        "Turret Weapons/Bay Weapons check",
        "Missile to-hit roll",
    ],
    "Table: Sample Diseases": ["Disease", "DM", "Damage", "Interval"],
    "Table: UWP Values for Trade Codes": [
        "Classification",
        "Code",
        "Size",
        "Atmos.",
        "Hydro",
        "Pop.",
        "Gov.",
        "Law",
        "TL",
    ],
    "Table: Psionic Range Costs": [
        "Range",
        "Distance to Target",
        "Clairvoyance",
        "Telekinesis",
        "Telepathy",
        "Teleportation",
    ],
    "Table: Pseudo-Hexadecimal Notation": [
        "Actual Value",
        "PseudoHex",
        "Actual Value",
        "PseudoHex",
        "Actual Value",
        "PseudoHex",
    ],
}


@dataclass
class Word:
    page: int
    flow: int
    block: int
    line: int
    number: int
    left: float
    top: float
    width: float
    height: float
    text: str

    @property
    def right(self) -> float:
        return self.left + self.width

    @property
    def bottom(self) -> float:
        return self.top + self.height


@dataclass
class Line:
    page: int
    flow: int
    block: int
    number: int
    words: list[Word]

    @property
    def left(self) -> float:
        return min(word.left for word in self.words)

    @property
    def right(self) -> float:
        return max(word.right for word in self.words)

    @property
    def top(self) -> float:
        return min(word.top for word in self.words)

    @property
    def bottom(self) -> float:
        return max(word.bottom for word in self.words)

    @property
    def height(self) -> float:
        return max(word.height for word in self.words)

    @property
    def text(self) -> str:
        return " ".join(word.text for word in sorted(self.words, key=lambda word: word.left))


@dataclass
class Block:
    page: int
    flow: int
    number: int
    lines: list[Line]

    @property
    def key(self) -> tuple[int, int]:
        return self.flow, self.number

    @property
    def left(self) -> float:
        return min(line.left for line in self.lines)

    @property
    def right(self) -> float:
        return max(line.right for line in self.lines)

    @property
    def top(self) -> float:
        return min(line.top for line in self.lines)

    @property
    def bottom(self) -> float:
        return max(line.bottom for line in self.lines)

    @property
    def width(self) -> float:
        return self.right - self.left

    @property
    def height(self) -> float:
        return max(line.height for line in self.lines)

    @property
    def text(self) -> str:
        return join_wrapped_lines([line.text for line in self.lines])


@dataclass
class TableTitle:
    page: int
    top: float
    left: float
    right: float
    text: str
    source_line: Line
    scope_left: float = PAGE_LEFT
    scope_right: float = PAGE_RIGHT

    @property
    def center(self) -> float:
        return (self.left + self.right) / 2


@dataclass
class Table:
    title: TableTitle
    blocks: list[Block] = field(default_factory=list)
    lines: list[Line] = field(default_factory=list)
    column_starts: list[float] = field(default_factory=list)
    row_anchors: list[float] = field(default_factory=list)
    row_ranges: list[tuple[float, float]] = field(default_factory=list)
    header: list[str] = field(default_factory=list)
    rows: list[list[str]] = field(default_factory=list)

    @property
    def left(self) -> float:
        return self.title.scope_left

    @property
    def right(self) -> float:
        return self.title.scope_right

    @property
    def top(self) -> float:
        return self.title.top


@dataclass
class RenderItem:
    top: float
    left: float
    right: float
    markdown: str
    heading: str | None = None
    source_keys: set[tuple[int, int]] = field(default_factory=set)

    @property
    def width(self) -> float:
        return self.right - self.left

    @property
    def center(self) -> float:
        return (self.left + self.right) / 2


@dataclass(frozen=True)
class ShadeBand:
    left: float
    right: float
    top: float
    bottom: float

    @property
    def center_x(self) -> float:
        return (self.left + self.right) / 2

    @property
    def center_y(self) -> float:
        return (self.top + self.bottom) / 2


def run_poppler() -> str:
    if not PDF.exists():
        raise SystemExit(f"Missing source PDF: {PDF}")
    proc = subprocess.run(
        [
            "pdftotext",
            "-f",
            str(min(doc.physical_start for doc in DOCUMENTS)),
            "-l",
            str(max(doc.physical_end for doc in DOCUMENTS)),
            "-tsv",
            str(PDF),
            "-",
        ],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return proc.stdout


def detect_shade_bands(image_path: Path) -> list[ShadeBand]:
    """Locate the gray rectangles used for alternating table rows."""
    image = np.asarray(Image.open(image_path).convert("L"))
    height, width = image.shape
    values, counts = np.unique(image, return_counts=True)
    gray_candidates = [
        (int(count), int(value))
        for value, count in zip(values, counts)
        if 150 <= int(value) <= 225
    ]
    if not gray_candidates:
        return []
    minimum_fill = max(300, int(image.size * 0.003))
    table_grays = [
        value
        for count, value in gray_candidates
        if count >= minimum_fill
    ]
    if not table_grays:
        return []
    image16 = image.astype(np.int16)
    mask = np.zeros_like(image, dtype=bool)
    for table_gray in table_grays:
        mask |= np.abs(image16 - table_gray) <= 2

    bands: list[ShadeBand] = []
    labels, _ = ndimage.label(mask)
    for component in ndimage.find_objects(labels):
        if component is None:
            continue
        y_slice, x_slice = component
        y0, y1 = y_slice.start, y_slice.stop
        x0, x1 = x_slice.start, x_slice.stop
        if y1 - y0 < 4 or x1 - x0 < 55:
            continue
        bands.append(ShadeBand(float(x0), float(x1), float(y0), float(y1)))
    return sorted(bands, key=lambda band: (band.top, band.left))


def render_shading() -> dict[int, list[ShadeBand]]:
    first = min(doc.physical_start for doc in DOCUMENTS)
    last = max(doc.physical_end for doc in DOCUMENTS)
    detected: dict[int, list[ShadeBand]] = {}
    with tempfile.TemporaryDirectory(prefix="cepod-shading-") as temp:
        prefix = Path(temp) / "page"
        subprocess.run(
            [
                "pdftoppm",
                "-f",
                str(first),
                "-l",
                str(last),
                "-r",
                "72",
                "-gray",
                str(PDF),
                str(prefix),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        for path in Path(temp).glob("page-*.pgm"):
            match = re.search(r"-(\d+)\.pgm$", path.name)
            if match:
                detected[int(match.group(1))] = detect_shade_bands(path)
    return detected


def parse_words(tsv: str) -> dict[int, list[Word]]:
    pages: dict[int, list[Word]] = defaultdict(list)
    for row in csv.DictReader(io.StringIO(tsv), delimiter="\t"):
        if row["level"] != "5":
            continue
        top = float(row["top"])
        if top >= FOOTER_TOP:
            continue
        word = Word(
            page=int(row["page_num"]),
            flow=int(row["par_num"]),
            block=int(row["block_num"]),
            line=int(row["line_num"]),
            number=int(row["word_num"]),
            left=float(row["left"]),
            top=top,
            width=float(row["width"]),
            height=float(row["height"]),
            text=row["text"],
        )
        pages[word.page].append(word)
    return pages


def make_lines(words: Iterable[Word]) -> list[Line]:
    grouped: dict[tuple[int, int, int], list[Word]] = defaultdict(list)
    for word in words:
        grouped[(word.flow, word.block, word.line)].append(word)
    lines = [
        Line(group[0].page, key[0], key[1], key[2], sorted(group, key=lambda word: word.number))
        for key, group in grouped.items()
    ]
    return sorted(lines, key=lambda line: (line.top, line.left))


def make_blocks(lines: Iterable[Line]) -> list[Block]:
    grouped: dict[tuple[int, int], list[Line]] = defaultdict(list)
    for line in lines:
        grouped[(line.flow, line.block)].append(line)
    blocks = [
        Block(group[0].page, key[0], key[1], sorted(group, key=lambda line: (line.top, line.left)))
        for key, group in grouped.items()
    ]
    return sorted(blocks, key=lambda block: (block.top, block.left))


def coalesce_inline_lines(lines: list[Line]) -> list[Line]:
    """Rejoin prose fragments that Poppler split on unusually wide spaces."""
    result: list[Line] = []
    for group in group_by_coordinate(lines, lambda line: line.top, 1.5):
        current: Line | None = None
        for line in sorted(group, key=lambda item: item.left):
            same_column = (
                current is not None
                and ((current.left + current.right) / 2 < PAGE_MID)
                == ((line.left + line.right) / 2 < PAGE_MID)
            )
            if (
                current is not None
                and same_column
                and current.height < 18
                and line.height < 18
                and -2 <= line.left - current.right <= 30
            ):
                current = Line(
                    current.page,
                    current.flow,
                    current.block,
                    current.number,
                    sorted(current.words + line.words, key=lambda word: word.left),
                )
                result[-1] = current
            else:
                current = line
                result.append(line)
    return sorted(result, key=lambda line: (line.top, line.left))


def join_wrapped_lines(lines: list[str]) -> str:
    result = ""
    for raw_line in lines:
        line = re.sub(r"\s+", " ", raw_line).strip()
        if not line:
            continue
        if not result:
            result = line
        elif result.endswith("-") and line[:1].islower():
            result = result[:-1] + line
        else:
            result += " " + line
    return result


def markdown_escape(text: str) -> str:
    return (
        text.replace("\\", "\\\\")
        .replace("|", "\\|")
        .replace("<", "\\<")
        .replace(">", "\\>")
    )


def slugify(text: str) -> str:
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
    text = text.lower().strip()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[\s_-]+", "-", text)
    return text.strip("-")


def split_table_titles(lines: list[Line]) -> list[TableTitle]:
    titles: list[TableTitle] = []
    for line in lines:
        indices = [index for index, word in enumerate(line.words) if word.text == "Table:"]
        for position, start in enumerate(indices):
            stop = indices[position + 1] if position + 1 < len(indices) else len(line.words)
            prefix = start - 1 if start and line.words[start - 1].text == "Example" else start
            words = line.words[prefix:stop]
            if not words:
                continue
            text = " ".join(word.text for word in words)
            titles.append(
                TableTitle(
                    page=line.page,
                    top=min(word.top for word in words),
                    left=min(word.left for word in words),
                    right=max(word.right for word in words),
                    text=text,
                    source_line=line,
                )
            )

    # In a few wide layouts InDesign placed the remainder of a bold table
    # title in an adjacent text object on the same visual baseline.
    for title in titles:
        candidates = [
            line
            for line in lines
            if line is not title.source_line
            and abs(line.top - title.top) <= 5
            and 0 <= line.left - title.right <= 7
            and "Table:" not in line.text
            and len(line.words) <= 5
        ]
        if candidates:
            continuation = min(candidates, key=lambda line: line.left - title.right)
            title.text = f"{title.text} {continuation.text}"
            title.right = max(title.right, continuation.right)
    assign_table_scopes(titles, lines)
    return sorted(titles, key=lambda title: (title.top, title.left))


def assign_table_scopes(titles: list[TableTitle], lines: list[Line]) -> None:
    same_row_groups: list[list[TableTitle]] = []
    for title in sorted(titles, key=lambda item: (item.top, item.left)):
        if same_row_groups and abs(same_row_groups[-1][0].top - title.top) <= 3:
            same_row_groups[-1].append(title)
        else:
            same_row_groups.append([title])

    for group in same_row_groups:
        group.sort(key=lambda item: item.center)
        if len(group) > 1:
            centers = [item.center for item in group]
            for index, title in enumerate(group):
                title.scope_left = PAGE_LEFT if index == 0 else (centers[index - 1] + centers[index]) / 2
                title.scope_right = PAGE_RIGHT if index == len(group) - 1 else (centers[index] + centers[index + 1]) / 2
            continue

        title = group[0]
        if title.right < PAGE_MID - 4:
            title.scope_left, title.scope_right = PAGE_LEFT, PAGE_MID - 5
        elif title.left > PAGE_MID + 4:
            title.scope_left, title.scope_right = PAGE_MID + 5, PAGE_RIGHT
        else:
            title.scope_left, title.scope_right = PAGE_LEFT, PAGE_RIGHT

        # Some full-width tables have a left-aligned title.  A header row
        # spanning nearly the whole page is a strong signal to widen it.
        nearby = [
            line
            for line in lines
            if title.top + 8 <= line.top <= title.top + 35
            and len(line.words) <= 8
        ]
        if (
            title.scope_right < PAGE_MID
            and nearby
            and min(line.left for line in nearby) < 80
            and max(line.right for line in nearby) > 455
        ):
            title.scope_left, title.scope_right = PAGE_LEFT, PAGE_RIGHT


def group_by_coordinate(items: list, coordinate, tolerance: float) -> list[list]:
    groups: list[list] = []
    for item in sorted(items, key=lambda value: (coordinate(value), getattr(value, "left", 0))):
        if groups and abs(coordinate(groups[-1][0]) - coordinate(item)) <= tolerance:
            groups[-1].append(item)
        else:
            groups.append([item])
    return groups


def match_titles_to_header_bands(
    titles: list[TableTitle],
    bands: list[ShadeBand],
    pending_titles: list[TableTitle],
) -> tuple[list[tuple[TableTitle, ShadeBand]], list[TableTitle]]:
    """Match bold `Table:` titles to the first shaded row under each title."""
    matches: list[tuple[TableTitle, ShadeBand]] = []
    used: set[ShadeBand] = set()
    band_groups = group_by_coordinate(bands, lambda band: band.top, 3.0)

    ordered_titles = [
        title
        for group in group_by_coordinate(titles, lambda item: item.top, 5.0)
        for title in sorted(group, key=lambda item: item.center)
    ]
    for title in ordered_titles:
        candidates = [
            band
            for band in bands
            if band not in used
            and 0 <= band.top - title.top <= 55
            and band.left - 25 <= title.center <= band.right + 25
            and not any(
                abs(band.left - prior.left) <= 14
                and abs(band.right - prior.right) <= 14
                and 0 < band.top - prior.top < 80
                for prior in used
            )
        ]
        if not candidates:
            continue
        band = min(
            candidates,
            key=lambda item: (
                item.top - title.top,
                abs(item.center_x - title.center),
            ),
        )
        matches.append((title, band))
        used.add(band)

    matched_title_ids = {id(title) for title, _ in matches}
    unmatched = [title for title in titles if id(title) not in matched_title_ids]

    # A title printed at the foot of one page can describe a table that starts
    # at the top of the next page.  Match such pending titles before treating
    # an unlabelled top-of-page table as a continuation.
    unclaimed_top_groups = [
        group
        for group in band_groups
        if min(band.top for band in group) < 180
        and any(band not in used for band in group)
    ]
    carried = pending_titles + unmatched
    if pending_titles and unclaimed_top_groups:
        group = min(unclaimed_top_groups, key=lambda values: min(item.top for item in values))
        available = sorted((band for band in group if band not in used), key=lambda band: band.center_x)
        for title, band in zip(pending_titles, available):
            synthetic = TableTitle(
                page=title.page + 1,
                top=max(0.0, band.top - 20),
                left=band.left,
                right=band.right,
                text=title.text,
                source_line=title.source_line,
                scope_left=band.left,
                scope_right=band.right,
            )
            matches.append((synthetic, band))
            used.add(band)
            if title in carried:
                carried.remove(title)
    return matches, carried


def shaded_table(
    title: TableTitle,
    header_band: ShadeBand,
    all_header_bands: set[ShadeBand],
    bands: list[ShadeBand],
    lines: list[Line],
    blocks: list[Block],
) -> Table:
    if re.search(r"\b(?:by|of|for|and|Ranged)$", title.text, re.IGNORECASE):
        continuations = [
            line
            for line in lines
            if title.top + 3 <= line.top < header_band.top
            and header_band.left <= (line.left + line.right) / 2 <= header_band.right
            and len(line.words) <= 4
            and "Table:" not in line.text
        ]
        if continuations:
            continuation = min(continuations, key=lambda line: line.top)
            title.text = f"{title.text} {continuation.text}"

    title.scope_left = header_band.left
    title.scope_right = header_band.right

    later_header_tops = [
        band.top
        for band in all_header_bands
        if band.top > header_band.top + 2
        and horizontal_overlap(header_band.left, header_band.right, band.left, band.right) > 10
    ]
    stop = min(later_header_tops + [FOOTER_TOP])
    row_bands = [
        band
        for band in bands
        if header_band.top <= band.top < stop
        and abs(band.left - header_band.left) <= 14
        and abs(band.right - header_band.right) <= 14
    ]
    row_bands.sort(key=lambda band: band.top)

    anchors: list[float] = []
    row_ranges: list[tuple[float, float]] = []
    for index, band in enumerate(row_bands):
        if index:
            previous = row_bands[index - 1]
            anchors.append((previous.bottom + band.top) / 2)
            row_ranges.append((previous.bottom, band.top))
        anchors.append(band.center_y)
        row_ranges.append((band.top, band.bottom))

    last = row_bands[-1]
    block_by_key = {block.key: block for block in blocks}
    following = [
        line
        for line in lines
        if last.bottom - 1 <= line.top < min(stop, last.bottom + 55)
        and horizontal_overlap(header_band.left, header_band.right, line.left, line.right) > 1
        and not is_heading_line(line)
        and not line.text.startswith("Table:")
        and not block_is_prose(
            block_by_key[(line.flow, line.block)],
            header_band.left,
            header_band.right,
        )
    ]
    region_bottom = last.bottom
    if following:
        first_y = min(line.top for line in following)
        if first_y <= last.bottom + 24:
            final_stops = [
                line.top
                for line in lines
                if line.top > first_y + 2
                and (
                    is_heading_line(line)
                    or block_is_prose(
                        block_by_key[(line.flow, line.block)],
                        header_band.left,
                        header_band.right,
                    )
                )
            ]
            final_stop = min(final_stops + [min(stop, first_y + 70)])
            final_lines = [
                line
                for line in lines
                if first_y <= line.top < final_stop
                and horizontal_overlap(
                    header_band.left,
                    header_band.right,
                    line.left,
                    line.right,
                )
                > 1
                and not is_heading_line(line)
                and not line.text.startswith("Table:")
            ]
            if final_lines:
                final_bottom = min(
                    max(line.bottom for line in final_lines) + 2,
                    final_stop - 0.1,
                )
                anchors.append((last.bottom + max(line.bottom for line in final_lines)) / 2)
                row_ranges.append((last.bottom, final_bottom))
                region_bottom = final_bottom

    clipped: list[Line] = []
    for line in lines:
        if not (header_band.top - 2 <= line.top <= region_bottom):
            continue
        words = [
            word
            for word in line.words
            if header_band.left - 2 <= (word.left + word.right) / 2 <= header_band.right + 2
        ]
        if words:
            clipped.append(Line(line.page, line.flow, line.block, line.number, words))

    selected_blocks = [
        block
        for block in blocks
        if any(
            header_band.top - 2 <= line.top <= region_bottom
            and horizontal_overlap(header_band.left, header_band.right, line.left, line.right) > 1
            for line in block.lines
        )
    ]
    # Consume the printed title as well as the shaded grid.
    selected_blocks.extend(
        block
        for block in blocks
        if title.source_line.page == block.page
        and block.key == (title.source_line.flow, title.source_line.block)
    )
    unique_blocks = {block.key: block for block in selected_blocks}
    table = Table(
        title=title,
        blocks=list(unique_blocks.values()),
        lines=clipped,
        row_anchors=anchors,
        row_ranges=row_ranges,
    )
    table_to_grid(table)
    return table


def career_tables(
    printed_page: int,
    lines: list[Line],
    blocks: list[Block],
) -> list[Table]:
    """Restore the four untitled career matrices printed on pages 20–27."""
    careers = CAREER_GROUPS.get(printed_page)
    if not careers:
        return []

    words = [word for line in lines for word in line.words]

    def token(text: str) -> str:
        return re.sub(r"[^a-z0-9]+", "", text.casefold())

    career_starts: list[float] = []
    previous = PAGE_LEFT + 20
    for career in careers:
        first = token(career.split()[0])
        candidates = [
            word
            for word in words
            if word.top < 105
            and word.left > previous + 8
            and token(word.text) == first
        ]
        if not candidates:
            return []
        selected = min(candidates, key=lambda word: word.left)
        career_starts.append(selected.left)
        previous = selected.left

    first_column = min(
        (word.left for word in words if word.left < career_starts[0] - 5),
        default=PAGE_LEFT,
    )
    column_starts = [first_column, *career_starts]
    left_limit = career_starts[0] - 5

    if printed_page % 2 == 0:
        definitions = [
            (
                "Career Qualifications",
                "Requirement",
                ["Qualifications", "Survival", "Commission", "Advancement", "Re-enlistment"],
                None,
            ),
            ("Career Ranks and Skills", "Rank", [str(value) for value in range(7)], "Ranks"),
            ("Career Material Benefits", "Roll", [str(value) for value in range(1, 8)], "Material"),
            ("Career Cash Benefits", "Roll", [str(value) for value in range(1, 8)], "Cash"),
        ]
    else:
        definitions = [
            ("Personal Development Skills", "Roll", [str(value) for value in range(1, 7)], None),
            ("Service Skills", "Roll", [str(value) for value in range(1, 7)], "Service"),
            ("Specialist Skills", "Roll", [str(value) for value in range(1, 7)], "Specialist"),
            ("Advanced Education Skills", "Roll", [str(value) for value in range(1, 7)], "Adv"),
        ]

    section_tops: list[float] = []
    for _, _, _, marker in definitions:
        if marker is None:
            section_tops.append(min(word.top for word in words))
            continue
        candidates = [
            word
            for word in words
            if token(word.text) == token(marker)
        ]
        if not candidates:
            return []
        section_tops.append(min(word.top for word in candidates))
    section_bottoms = section_tops[1:] + [FOOTER_TOP]

    tables: list[Table] = []
    for definition, section_top, section_bottom in zip(
        definitions,
        section_tops,
        section_bottoms,
    ):
        title_text, first_header, labels, _ = definition
        anchors: list[float] = []
        for label in labels:
            matches = [
                word
                for word in words
                if section_top <= word.top < section_bottom
                and word.left < left_limit
                and token(word.text) == token(label)
            ]
            if not matches:
                anchors = []
                break
            anchors.append(min(matches, key=lambda word: word.top).top)
        if len(anchors) != len(labels):
            continue

        boundaries = [max(section_top, anchors[0] - 14)]
        boundaries.extend(
            (anchors[index] + anchors[index + 1]) / 2
            for index in range(len(anchors) - 1)
        )
        boundaries.append(section_bottom)

        grid: list[list[str]] = []
        for row_index in range(len(labels)):
            row_top = boundaries[row_index]
            row_bottom = boundaries[row_index + 1]
            cells: list[list[Word]] = [[] for _ in column_starts]
            for word in words:
                if not (row_top <= word.top < row_bottom):
                    continue
                valid = [
                    index
                    for index, start in enumerate(column_starts)
                    if start <= word.left + 12
                ]
                column = valid[-1] if valid else 0
                cells[column].append(word)

            rendered_row: list[str] = []
            for cell in cells:
                visual_lines = group_by_coordinate(cell, lambda word: word.top, 2.0)
                rendered_row.append(
                    join_wrapped_lines(
                        [
                            " ".join(
                                word.text
                                for word in sorted(group, key=lambda word: word.left)
                            )
                            for group in visual_lines
                        ]
                    )
                )
            grid.append(rendered_row)

        region_lines = [
            line for line in lines if section_top <= line.top < section_bottom
        ]
        title = TableTitle(
            page=lines[0].page,
            top=section_top,
            left=PAGE_LEFT,
            right=PAGE_RIGHT,
            text=f"Table: {title_text} — {careers[0]} to {careers[-1]}",
            source_line=region_lines[0],
            scope_left=PAGE_LEFT,
            scope_right=PAGE_RIGHT,
        )
        tables.append(
            Table(
                title=title,
                blocks=blocks,
                lines=region_lines,
                column_starts=column_starts,
                header=[first_header, *careers],
                rows=grid,
            )
        )
    return tables


def horizontal_overlap(left1: float, right1: float, left2: float, right2: float) -> float:
    return max(0.0, min(right1, right2) - max(left1, left2))


def is_heading_line(line: Line) -> bool:
    text = line.text.strip()
    return (
        line.height >= 16.0
        and len(text) <= 100
        and not text.startswith(("•", "-", "Table:"))
    )


def block_is_prose(block: Block, scope_left: float, scope_right: float) -> bool:
    scope_width = scope_right - scope_left
    words = sum(len(line.words) for line in block.lines)
    return (
        (block.width >= scope_width * 0.68 or block.width >= 230)
        and (
            block.left <= scope_left + 18
            or abs(block.left - 312) <= 18
            or abs(block.left - 45) <= 18
        )
        and len(block.lines) >= 2
        and words >= 14
    )


def infer_table_blocks(
    title: TableTitle,
    all_titles: list[TableTitle],
    blocks: list[Block],
) -> list[Block]:
    scope_left, scope_right = title.scope_left, title.scope_right
    later_stops = [
        other.top
        for other in all_titles
        if other is not title
        and other.top > title.top + 3
        and horizontal_overlap(scope_left, scope_right, other.scope_left, other.scope_right) > 20
    ]
    heading_stops = [
        line.top
        for block in blocks
        for line in block.lines
        if line.top > title.top + 8
        and is_heading_line(line)
        and horizontal_overlap(scope_left, scope_right, line.left, line.right) > 10
    ]
    hard_stop = min(later_stops + heading_stops + [FOOTER_TOP])

    candidates = [
        block
        for block in blocks
        if block.bottom > title.top + 5
        and block.top < hard_stop
        and horizontal_overlap(scope_left, scope_right, block.left, block.right) > 1
    ]
    candidates.sort(key=lambda block: (block.top, block.left))

    selected: list[Block] = []
    initial_limit = title.top + 42
    for block in candidates:
        if block.top > initial_limit:
            continue
        if any(is_heading_line(line) for line in block.lines):
            continue
        if block_is_prose(block, scope_left, scope_right) and block.top > title.top + 26:
            continue
        selected.append(block)

    if not selected:
        return []

    end = max(block.bottom for block in selected)
    changed = True
    while changed:
        changed = False
        for block in candidates:
            if block in selected or block.top > end + 8:
                continue
            if any(is_heading_line(line) for line in block.lines):
                continue
            if block_is_prose(block, scope_left, scope_right) and block.top > title.top + 30:
                continue
            selected.append(block)
            end = max(end, block.bottom)
            changed = True
    return sorted(selected, key=lambda block: (block.top, block.left))


def cluster_positions(values: list[float], tolerance: float = 7.0) -> list[tuple[float, int]]:
    if not values:
        return []
    clusters: list[list[float]] = []
    for value in sorted(values):
        if clusters and value - sum(clusters[-1]) / len(clusters[-1]) <= tolerance:
            clusters[-1].append(value)
        else:
            clusters.append([value])
    return [(sum(cluster) / len(cluster), len(cluster)) for cluster in clusters]


def infer_column_starts(table: Table) -> list[float]:
    values: list[float] = []
    for line in table.lines:
        if line.top <= table.title.top + 4:
            continue
        values.append(line.left)
    clusters = cluster_positions(values)
    kept = [(position, count) for position, count in clusters if count >= 2]
    if not kept:
        kept = clusters[:]

    # Prefer frequently recurring starts and keep columns a useful distance
    # apart. Dense numerical tables legitimately have narrow columns.
    pruned: list[tuple[float, int]] = []
    for position, count in kept:
        if not pruned or position - pruned[-1][0] >= 18:
            pruned.append((position, count))
        elif count > pruned[-1][1]:
            pruned[-1] = (position, count)

    if len(pruned) > 10:
        strongest = sorted(pruned, key=lambda item: item[1], reverse=True)[:10]
        pruned = sorted(strongest)
    starts = [position for position, _ in pruned]

    override = HEADER_OVERRIDES.get(table.title.text)
    if override and table.row_ranges:
        header_words = sorted(
            (
                word
                for line in table.lines
                if table.row_ranges[0][0] <= line.top < table.row_ranges[0][1]
                for word in line.words
            ),
            key=lambda word: (word.left, word.top),
        )

        def normalized_token(text: str) -> str:
            return re.sub(r"[^a-z0-9]+", "", text.casefold())

        schema_starts: list[float] = []
        previous = table.title.scope_left - 10
        missing_first = False
        for index, label in enumerate(override):
            first_token = normalized_token(label.split()[0])
            candidates = [
                word
                for word in header_words
                if normalized_token(word.text) == first_token
                and word.left > previous + 5
            ]
            if not candidates:
                if index == 0:
                    missing_first = True
                    continue
                schema_starts = []
                break
            selected = min(candidates, key=lambda word: word.left)
            schema_starts.append(selected.left)
            previous = selected.left

        if missing_first and len(schema_starts) == len(override) - 1:
            body_left = min(
                (
                    word.left
                    for line in table.lines
                    if line.top >= table.row_ranges[1][0]
                    for word in line.words
                ),
                default=table.title.scope_left,
            )
            if body_left < schema_starts[0] - 5:
                schema_starts.insert(0, body_left)
        if len(schema_starts) == len(override):
            starts = schema_starts

    if not starts:
        starts = [table.title.scope_left]
    return starts


def nearest_column(x: float, starts: list[float]) -> int:
    return min(range(len(starts)), key=lambda index: abs(starts[index] - x))


def line_pieces(
    line: Line,
    starts: list[float],
    left_tolerance: float = 8,
) -> list[tuple[int, float, str]]:
    grouped: dict[int, list[Word]] = defaultdict(list)
    for word in sorted(line.words, key=lambda item: item.left):
        # Numeric table cells are often right-aligned, so a long value may
        # begin a few points to the left of the column's dominant start.
        valid = [
            index
            for index, start in enumerate(starts)
            if start <= word.left + left_tolerance
        ]
        index = valid[-1] if valid else 0
        grouped[index].append(word)
    return [
        (index, line.top, " ".join(word.text for word in words))
        for index, words in sorted(grouped.items())
    ]


def unique_y(values: Iterable[float], tolerance: float = 2.0) -> list[float]:
    clusters = cluster_positions(list(values), tolerance)
    return [position for position, _ in clusters]


def table_to_grid(table: Table) -> None:
    if not table.lines:
        clipped_lines: list[Line] = []
        for block in table.blocks:
            for line in block.lines:
                words = [
                    word
                    for word in line.words
                    if table.title.scope_left <= (word.left + word.right) / 2 <= table.title.scope_right
                ]
                if line.top > table.title.top + 4 and words:
                    clipped_lines.append(
                        Line(line.page, line.flow, line.block, line.number, words)
                    )
        table.lines = clipped_lines
    table.column_starts = infer_column_starts(table)
    pieces: list[tuple[int, float, str]] = []
    for line in table.lines:
        is_header = bool(
            table.row_ranges
            and table.row_ranges[0][0] <= line.top < table.row_ranges[0][1]
        )
        pieces.extend(
            line_pieces(
                line,
                table.column_starts,
                left_tolerance=20 if is_header else 16,
            )
        )
    if not pieces:
        table.header = ["Value"]
        table.rows = []
        return

    by_column: dict[int, list[tuple[float, str]]] = defaultdict(list)
    by_y_columns: dict[float, set[int]] = defaultdict(set)
    y_values = unique_y(top for _, top, _ in pieces)
    for column, top, text in pieces:
        y = min(y_values, key=lambda value: abs(value - top))
        by_column[column].append((y, text))
        by_y_columns[y].add(column)

    usable_columns = [
        column for column, entries in by_column.items() if len(entries) >= 2
    ]
    if usable_columns:
        anchor_column = min(
            usable_columns,
            key=lambda column: (len(by_column[column]), column),
        )
    else:
        anchor_column = min(by_column)

    if table.row_anchors:
        anchors = table.row_anchors
    else:
        anchors = unique_y(top for top, _ in by_column[anchor_column])
        for y, columns in by_y_columns.items():
            if len(columns) >= 2 and all(abs(y - anchor) > 2 for anchor in anchors):
                anchors.append(y)
        anchors.sort()

        filtered: list[float] = []
        for y in anchors:
            support = len(by_y_columns.get(y, set()))
            if filtered and y - filtered[-1] < 15.2 and support < 2:
                continue
            filtered.append(y)
        anchors = filtered or [min(y_values)]

    cells: list[list[list[str]]] = [
        [[] for _ in table.column_starts] for _ in anchors
    ]
    for column, top, text in pieces:
        contained = [
            index
            for index, (start, end) in enumerate(table.row_ranges)
            if start <= top < end
        ]
        row = (
            contained[0]
            if contained
            else min(range(len(anchors)), key=lambda index: abs(anchors[index] - top))
        )
        cells[row][column].append(text)

    rows = [
        [join_wrapped_lines(cell) for cell in row]
        for row in cells
    ]
    table.header = rows[0]
    table.rows = rows[1:]

    # GFM needs a meaningful header row.  Supply neutral names only when the
    # source table has genuinely blank header cells.
    table.header = [
        value if value else f"Column {index + 1}"
        for index, value in enumerate(table.header)
    ]
    override = HEADER_OVERRIDES.get(table.title.text)
    if override and len(override) == len(table.header):
        table.header = override
    table.rows = [
        row for row in table.rows if any(value.strip() for value in row)
    ]


def render_table(table: Table) -> str:
    title = re.sub(r"\s+", " ", table.title.text).strip()
    lines = [f"### {markdown_escape(title)}", ""]
    header = [markdown_escape(value) for value in table.header]
    lines.append("| " + " | ".join(header) + " |")
    lines.append("| " + " | ".join("---" for _ in header) + " |")
    for row in table.rows:
        padded = row + [""] * (len(header) - len(row))
        lines.append("| " + " | ".join(markdown_escape(value) for value in padded[: len(header)]) + " |")
    return "\n".join(lines)


def split_list_items(lines: list[Line]) -> list[str] | None:
    markers = re.compile(r"^(?:[•▪●]|(?:\d+|[a-zA-Z])[.)])\s+")
    items: list[str] = []
    current = ""
    found = False
    for line in lines:
        text = line.text.strip()
        if markers.match(text):
            found = True
            if current:
                items.append(current)
            current = text
        elif current:
            if current.endswith("-") and text[:1].islower():
                current = current[:-1] + text
            else:
                current += " " + text
        else:
            current = text
    if current:
        items.append(current)
    return items if found else None


def render_block(block: Block, document: Document) -> tuple[str, str | None]:
    text = block.text.strip()
    if not text:
        return "", None

    upper_title = text.upper()
    if (
        upper_title in {
            "INTRODUCTION",
            "LEGAL",
            document.title.upper(),
            document.book.upper() if document.book else "",
        }
        or re.fullmatch(r"CHAPTER \d+:.*", upper_title)
        or re.fullmatch(r"BOOK (ONE|TWO|THREE):.*", upper_title)
    ):
        return "", None

    if block.height >= 16.0 and len(text) <= 120:
        return f"### {markdown_escape(text)}", text

    items = split_list_items(block.lines)
    if items:
        rendered: list[str] = []
        for item in items:
            match = re.match(r"^([•▪●]|(?:\d+|[a-zA-Z])[.)])\s+(.*)", item)
            if not match:
                rendered.append(markdown_escape(item))
                continue
            marker, body = match.groups()
            if marker[0].isdigit() and marker.endswith("."):
                prefix = marker
            else:
                prefix = "-"
                if marker[0].isalnum() and not marker[0].isdigit():
                    body = f"{marker} {body}"
            rendered.append(f"{prefix} {markdown_escape(body)}")
        return "\n".join(rendered), None

    return markdown_escape(text), None


def order_items(items: list[RenderItem]) -> list[RenderItem]:
    if not items:
        return []
    left = [item for item in items if item.center < PAGE_MID and item.width < 350]
    right = [item for item in items if item.center >= PAGE_MID and item.width < 350]
    full = [item for item in items if item.width >= 350]

    if not left or not right:
        return sorted(items, key=lambda item: (item.top, item.left))

    ordered: list[RenderItem] = []
    remaining = set(range(len(items)))
    previous = -1.0
    for barrier_index in sorted(
        (index for index, item in enumerate(items) if item in full),
        key=lambda index: items[index].top,
    ):
        barrier = items[barrier_index]
        band = [
            (index, item)
            for index, item in enumerate(items)
            if index in remaining and previous <= item.top < barrier.top
        ]
        ordered.extend(
            item for _, item in sorted(
                band,
                key=lambda pair: (
                    0 if pair[1].center < PAGE_MID else 1,
                    pair[1].top,
                    pair[1].left,
                ),
            )
        )
        remaining.difference_update(index for index, _ in band)
        if barrier_index in remaining:
            ordered.append(barrier)
            remaining.remove(barrier_index)
        previous = barrier.top

    tail = [(index, items[index]) for index in remaining]
    ordered.extend(
        item for _, item in sorted(
            tail,
            key=lambda pair: (
                0 if pair[1].center < PAGE_MID else 1,
                pair[1].top,
                pair[1].left,
            ),
        )
    )
    return ordered


def build_page(
    physical_page: int,
    document: Document,
    page_words: list[Word],
    shade_bands: list[ShadeBand],
    pending_titles: list[TableTitle],
) -> tuple[str, list[str], list[TableTitle]]:
    printed_page = physical_page - 1
    lines = make_lines(page_words)
    blocks = make_blocks(lines)
    titles = split_table_titles(lines)

    tables = career_tables(printed_page, lines, blocks)
    consumed_words: set[int] = set()
    for table in tables:
        consumed_words.update(id(word) for line in table.lines for word in line.words)
    for title in titles:
        consumed_words.update(
            id(word)
            for word in title.source_line.words
            if title.left - 2 <= (word.left + word.right) / 2 <= title.right + 2
        )
    shade_matches, carried_titles = match_titles_to_header_bands(
        titles,
        shade_bands,
        pending_titles,
    )
    header_bands = {band for _, band in shade_matches}
    for title, header_band in shade_matches:
        table = shaded_table(
            title,
            header_band,
            header_bands,
            shade_bands,
            lines,
            blocks,
        )
        tables.append(table)
        consumed_words.update(id(word) for line in table.lines for word in line.words)
        if table.row_ranges:
            header_top = table.row_ranges[0][0]
            consumed_words.update(
                id(word)
                for line in lines
                if table.title.top - 2 <= line.top < header_top
                for word in line.words
                if table.left - 2
                <= (word.left + word.right) / 2
                <= table.right + 2
            )

    # Fall back to text geometry only on pages whose gray fills could not be
    # rendered or detected.  Shaded-but-unmatched titles are intentionally
    # carried to the following page for cross-page tables.
    if not shade_bands:
        carried_titles = []
        for title in titles:
            selected = infer_table_blocks(title, titles, blocks)
            if not selected:
                continue
            table = Table(title=title, blocks=selected)
            table_to_grid(table)
            tables.append(table)
            consumed_words.update(
                id(word)
                for block in selected
                for line in block.lines
                for word in line.words
            )

    # A PDF text object can contain both a table and adjacent prose.  Consume
    # only the words geometrically inside the table, then rebuild the blocks
    # from whatever remains instead of discarding the complete text object.
    remaining_lines: list[Line] = []
    for line in lines:
        words = [word for word in line.words if id(word) not in consumed_words]
        if words:
            remaining_lines.append(
                Line(
                    line.page,
                    line.flow,
                    line.block,
                    line.number,
                    words,
                )
            )
    remaining_blocks = make_blocks(coalesce_inline_lines(remaining_lines))

    items: list[RenderItem] = []
    headings: list[str] = []
    for block in remaining_blocks:
        markdown, heading = render_block(block, document)
        if not markdown:
            continue
        items.append(
            RenderItem(
                top=block.top,
                left=block.left,
                right=block.right,
                markdown=markdown,
                heading=heading,
                source_keys={block.key},
            )
        )
        if heading:
            headings.append(heading)

    for table in tables:
        items.append(
            RenderItem(
                top=table.top,
                left=table.left,
                right=table.right,
                markdown=render_table(table),
                heading=table.title.text,
                source_keys={block.key for block in table.blocks},
            )
        )
        headings.append(table.title.text)

    body = [f"## Page {printed_page}", ""]
    for item in order_items(items):
        body.extend([item.markdown, ""])
    return "\n".join(body).rstrip() + "\n", headings, carried_titles


def page_target(page: int, current: Document) -> str | None:
    target = PAGE_TO_DOCUMENT.get(page)
    if not target:
        return None
    filename = "" if target.filename == current.filename else target.filename
    return f"{filename}#page-{page}"


def chapter_target(number: int, current: Document) -> str | None:
    target = CHAPTER_BY_NUMBER.get(number)
    if not target:
        return None
    return "" if target.filename == current.filename else target.filename


def protect_links(text: str, replacements: list[tuple[re.Pattern[str], callable]]) -> str:
    saved: list[str] = []
    for pattern, make_link in replacements:
        def replace(match: re.Match[str]) -> str:
            link = make_link(match)
            if not link:
                return match.group(0)
            token = f"\u0000LINK{len(saved)}\u0000"
            saved.append(link)
            return token
        text = pattern.sub(replace, text)
    for index, link in enumerate(saved):
        text = text.replace(f"\u0000LINK{index}\u0000", link)
    return text


def linkify(text: str, document: Document, heading_targets: dict[str, tuple[str, str]]) -> str:
    replacements: list[tuple[re.Pattern[str], callable]] = []

    for number, chapter in CHAPTER_BY_NUMBER.items():
        full = re.compile(
            rf"\bChapter\s+{number}:\s+{re.escape(chapter.title.split(': ', 1)[1])}\b",
            re.IGNORECASE,
        )
        short = re.compile(rf"\bChapter\s+{number}\b", re.IGNORECASE)
        target = chapter_target(number, document)
        replacements.append(
            (full, lambda match, target=target: f"[{match.group(0)}]({target})" if target is not None else "")
        )
        replacements.append(
            (short, lambda match, target=target: f"[{match.group(0)}]({target})" if target is not None else "")
        )

        chapter_name = chapter.title.split(": ", 1)[1]
        named = re.compile(rf"\b{re.escape(chapter_name)}\s+chapter\b", re.IGNORECASE)
        replacements.append(
            (named, lambda match, target=target: f"[{match.group(0)}]({target})" if target is not None else "")
        )

    page_pattern = re.compile(r"\bpage\s+(\d{1,3})\b", re.IGNORECASE)
    replacements.append(
        (
            page_pattern,
            lambda match: (
                f"[{match.group(0)}]({page_target(int(match.group(1)), document)})"
                if page_target(int(match.group(1)), document)
                else ""
            ),
        )
    )

    for normalized, (filename, anchor) in sorted(
        heading_targets.items(),
        key=lambda item: len(item[0]),
        reverse=True,
    ):
        if len(normalized) < 5 or normalized.lower().startswith("table:"):
            continue
        pattern = re.compile(
            rf"(?P<prefix>\bsee(?:\s+also)?\s+(?:the\s+)?)"
            rf"(?P<title>{re.escape(normalized)})"
            rf"(?P<suffix>\s+(?:section|rules))?",
            re.IGNORECASE,
        )
        target = ("" if filename == document.filename else filename) + f"#{anchor}"
        replacements.append(
            (
                pattern,
                lambda match, target=target: (
                    f"{match.group('prefix')}[{match.group('title')}"
                    f"{match.group('suffix') or ''}]({target})"
                ),
            )
        )

    rendered: list[str] = []
    for line in text.splitlines():
        # Headings define the stable GFM anchors used by the links.  Rewriting
        # their visible text as links would also change those anchors.
        if re.match(r"^#{1,6}\s+", line):
            rendered.append(line)
        else:
            rendered.append(protect_links(line, replacements))
    return "\n".join(rendered) + ("\n" if text.endswith("\n") else "")


def collect_heading_targets(
    headings_by_document: dict[str, list[str]],
) -> tuple[dict[str, tuple[str, str]], dict[str, list[tuple[str, str]]]]:
    occurrences: dict[str, list[tuple[str, str, str]]] = defaultdict(list)
    indexed: dict[str, list[tuple[str, str]]] = defaultdict(list)
    for document in DOCUMENTS:
        used: Counter[str] = Counter()
        for heading in headings_by_document[document.filename]:
            base = slugify(heading)
            if not base:
                continue
            count = used[base]
            used[base] += 1
            anchor = base if count == 0 else f"{base}-{count}"
            normalized = re.sub(r"\s+", " ", heading).strip()
            occurrences[normalized.casefold()].append((document.filename, anchor, normalized))
            indexed[document.filename].append((normalized, anchor))
    unique = {
        values[0][2]: (values[0][0], values[0][1])
        for values in occurrences.values()
        if len(values) == 1
    }
    return unique, indexed


def render_index(indexed: dict[str, list[tuple[str, str]]]) -> str:
    lines = [
        "# Cepheus Engine SRD — Markdown Edition",
        "",
        "This directory is a GitHub-Flavored Markdown conversion of "
        "[`cepodnew.pdf`](../cepodnew.pdf). It uses native Markdown headings, "
        "lists, links, and pipe tables.",
        "",
        "## Table of Contents",
        "",
        f"- [{DOCUMENTS[0].title}]({DOCUMENTS[0].filename}) — pages 3–10",
    ]

    for book in ("Book One: Characters", "Book Two: Starships and Interstellar Travel", "Book Three: Referees"):
        lines.extend(["", f"### {book}", ""])
        for document in DOCUMENTS:
            if document.book != book:
                continue
            lines.append(
                f"- [{document.title}]({document.filename}) — "
                f"pages {document.printed_start}–{document.printed_end}"
            )
    legal = next(document for document in DOCUMENTS if document.filename == "legal.md")
    lines.extend(["", "### Supplementary", "", f"- [{legal.title}]({legal.filename}) — pages 153–154"])

    topics: list[tuple[str, str, str]] = []
    tables: list[tuple[str, str, str]] = []
    for filename, entries in indexed.items():
        for heading, anchor in entries:
            if re.match(r"^(?:Example )?Table:", heading, re.IGNORECASE):
                tables.append((heading, filename, anchor))
                continue
            if (
                re.fullmatch(r"(?:BOOK|CHAPTER)\b.*", heading, re.IGNORECASE)
                or re.fullmatch(r"Page \d+", heading, re.IGNORECASE)
            ):
                continue
            topics.append((heading, filename, anchor))
    topics.sort(key=lambda item: (item[0].casefold(), item[1], item[2]))

    lines.extend(["", "## Topic Index", ""])
    current_letter = ""
    for heading, filename, anchor in topics:
        letter = heading[0].upper() if heading else "#"
        if not letter.isalpha():
            letter = "#"
        if letter != current_letter:
            current_letter = letter
            if lines[-1]:
                lines.append("")
            lines.extend([f"### {letter}", ""])
        lines.append(f"- [{heading}]({filename}#{anchor})")

    lines.extend(["", "## Table Index", ""])
    for heading, filename, anchor in sorted(
        tables,
        key=lambda item: (
            re.sub(r"^(?:Example )?Table:\s*", "", item[0]).casefold(),
            item[1],
            item[2],
        ),
    ):
        label = re.sub(r"^(?:Example )?Table:\s*", "", heading)
        lines.append(f"- [{label}]({filename}#{anchor})")
    lines.append("")
    return "\n".join(lines)


def apply_layout_corrections(filename: str, text: str) -> tuple[str, list[str]]:
    """Handle a handful of source tables whose last row is laid out elsewhere."""
    extra_headings: list[str] = []

    def replace_once(old: str, new: str) -> None:
        nonlocal text
        if old not in text:
            raise RuntimeError(f"{filename}: layout correction no longer matches: {old[:60]!r}")
        text = text.replace(old, new, 1)

    if filename == "06-off-world-travel.md":
        replace_once("T=2", "T = 2√(D/A)")
        replace_once(
            "| 13–14 | Life imprisonment |\n\n15+ Death",
            "| 13–14 | Life imprisonment |\n| 15+ | Death |",
        )

    if filename == "08-ship-design-and-construction.md":
        tail = "\n".join(
            f"| {code} | {plant} | {fuel} |"
            for code, plant, fuel in [
                ("sL", "4.5", "1.5"),
                ("sM", "5.1", "1.7"),
                ("sN", "5.7", "1.9"),
                ("sP", "6.3", "2.1"),
                ("sQ", "6.9", "2.3"),
                ("sR", "7.5", "2.5"),
                ("sS", "8.1", "2.7"),
                ("sT", "8.7", "2.9"),
                ("sU", "9.3", "3.1"),
                ("sV", "9.9", "3.3"),
                ("sW", "10.5", "3.5"),
            ]
        )
        replace_once("| sK | 3.9 | 1.3 |", f"| sK | 3.9 | 1.3 |\n{tail}")
        replace_once(
            "\n\nsL 4.5\n\n1.5\n\nsM 5.1\n\n1.7\n\nsN 5.7\n\n1.9"
            "\n\nsP 6.3\n\n2.1\n\nsQ 6.9\n\n2.3\n\nsR 7.5\n\n2.5"
            "\n\nsS 8.1\n\n2.7\n\nsT 8.7\n\n2.9\n\nsU 9.3\n\n3.1"
            "\n\nsV 9.9\n\n3.3\n\nsW 10.5 3.5",
            "",
        )

    if filename == "10-space-combat.md":
        replace_once(
            "| Very Long | 2 |\n\nDistant 2",
            "| Very Long | 2 |\n| Distant | 2 |",
        )
        replace_once(
            "| Succeeded With Effect 1–5 | 7+ |\n\nSucceeded With Effect 6+ 6+",
            "| Succeeded With Effect 1–5 | 7+ |\n"
            "| Succeeded With Effect 6+ | 6+ |",
        )

    if filename == "13-planetary-wilderness-encounters.md":
        replace_once(
            "15+ 5D6\n\n### Table: Animal Size",
            "### Table: Animal Size",
        )
        replace_once(
            "| 12–14 | 4D6 |\n\n## Page 134",
            "| 12–14 | 4D6 |\n| 15+ | 5D6 |\n\n## Page 134",
        )
        replace_once(
            "Thrasher melee (close quarters)\n\n### Creating Encounter Tables",
            "### Creating Encounter Tables",
        )
        replace_once(
            "| Teeth | melee (close quarters) |",
            "| Teeth | melee (close quarters) |\n"
            "| Thrasher | melee (close quarters) |",
        )
        replace_once(
            "1D6 Animal Encounter Table Template 1D6 Animal Type "
            "1 Scavenger 2 Herbivore 3 Herbivore 4 Herbivore 5 Omnivore 6 Carnivore",
            "### Table: 1D6 Animal Encounter Table Template\n\n"
            "| 1D6 | Animal Type |\n"
            "| --- | --- |\n"
            "| 1 | Scavenger |\n"
            "| 2 | Herbivore |\n"
            "| 3 | Herbivore |\n"
            "| 4 | Herbivore |\n"
            "| 5 | Omnivore |\n"
            "| 6 | Carnivore |",
        )
        replace_once(
            "2D6 Animal Encounter Table Template\n\n"
            "2D6 Result 2 Scavenger 3 Omnivore 4 Scavenger 5 Omnivore "
            "6 Herbivore 7 Herbivore 8 Herbivore 9 Carnivore 10 Event "
            "11 Carnivore 12 Carnivore",
            "### Table: 2D6 Animal Encounter Table Template\n\n"
            "| 2D6 | Result |\n"
            "| --- | --- |\n"
            "| 2 | Scavenger |\n"
            "| 3 | Omnivore |\n"
            "| 4 | Scavenger |\n"
            "| 5 | Omnivore |\n"
            "| 6 | Herbivore |\n"
            "| 7 | Herbivore |\n"
            "| 8 | Herbivore |\n"
            "| 9 | Carnivore |\n"
            "| 10 | Event |\n"
            "| 11 | Carnivore |\n"
            "| 12 | Carnivore |",
        )
        extra_headings.extend(
            [
                "Table: 1D6 Animal Encounter Table Template",
                "Table: 2D6 Animal Encounter Table Template",
            ]
        )

    if filename == "15-starship-encounters.md":
        text = re.sub(r"\nEncounter Type\n", "\n", text)

    return text, extra_headings


def validate(files: dict[str, str]) -> None:
    if any(re.search(r"<[A-Za-z!/][^>]*>", text) for text in files.values()):
        raise RuntimeError("Generated output contains embedded HTML")
    if any(re.search(r"^Column \d+$|^[•▪●]$", text, re.MULTILINE) for text in files.values()):
        raise RuntimeError("Generated output contains an unresolved table or list fragment")

    table_count = 0
    for filename, text in files.items():
        lines = text.splitlines()
        for index, line in enumerate(lines):
            if (
                not line.startswith("|")
                or not all(
                    cell.strip() == "---"
                    for cell in line.strip("|").split("|")
                )
            ):
                continue
            table_count += 1
            expected = len(re.findall(r"(?<!\\)\|", line))
            if index == 0 or len(re.findall(r"(?<!\\)\|", lines[index - 1])) != expected:
                raise RuntimeError(f"{filename}: malformed table header near line {index + 1}")
            row = index + 1
            while row < len(lines) and lines[row].startswith("|"):
                if len(re.findall(r"(?<!\\)\|", lines[row])) != expected:
                    raise RuntimeError(f"{filename}: malformed table row near line {row + 1}")
                row += 1
    if table_count != 194:
        raise RuntimeError(f"Expected 194 native Markdown tables, found {table_count}")

    anchors: dict[str, set[str]] = {}
    for filename, text in files.items():
        used: Counter[str] = Counter()
        values: set[str] = set()
        for heading in re.findall(r"^#{1,6}\s+(.+?)\s*$", text, re.MULTILINE):
            base = slugify(heading)
            count = used[base]
            used[base] += 1
            values.add(base if count == 0 else f"{base}-{count}")
        anchors[filename] = values

    link_pattern = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
    broken: list[str] = []
    for filename, text in files.items():
        for target in link_pattern.findall(text):
            if target.startswith("../"):
                continue
            target_file, _, anchor = target.partition("#")
            target_file = target_file or filename
            if target_file not in files:
                broken.append(f"{filename}: missing file {target_file}")
            elif anchor and anchor not in anchors[target_file]:
                broken.append(f"{filename}: missing anchor {target_file}#{anchor}")
    if broken:
        raise RuntimeError("Broken internal links:\n" + "\n".join(broken[:50]))

    for document in DOCUMENTS:
        text = files[document.filename]
        for page in range(document.printed_start, document.printed_end + 1):
            if f"## Page {page}" not in text:
                raise RuntimeError(f"{document.filename}: missing page {page}")


def main() -> None:
    pages = parse_words(run_poppler())
    shading = render_shading()
    raw_documents: dict[str, str] = {}
    headings_by_document: dict[str, list[str]] = defaultdict(list)

    for document in DOCUMENTS:
        parts = [f"# {document.title}", ""]
        if document.book:
            parts.extend([f"*{document.book} · Original pages {document.printed_start}–{document.printed_end}*", ""])
        else:
            parts.extend([f"*Original pages {document.printed_start}–{document.printed_end}*", ""])
        headings_by_document[document.filename].append(document.title)

        pending_titles: list[TableTitle] = []
        for physical_page in range(document.physical_start, document.physical_end + 1):
            body, headings, pending_titles = build_page(
                physical_page,
                document,
                pages.get(physical_page, []),
                shading.get(physical_page, []),
                pending_titles,
            )
            parts.append(body.rstrip())
            parts.append("")
            headings_by_document[document.filename].append(f"Page {physical_page - 1}")
            headings_by_document[document.filename].extend(headings)
        parts.extend(["[Table of Contents and Topic Index](index.md)", ""])
        text, extra_headings = apply_layout_corrections(
            document.filename,
            "\n".join(parts),
        )
        raw_documents[document.filename] = text
        headings_by_document[document.filename].extend(extra_headings)

    heading_targets, indexed = collect_heading_targets(headings_by_document)
    files = {
        filename: linkify(text, next(doc for doc in DOCUMENTS if doc.filename == filename), heading_targets)
        for filename, text in raw_documents.items()
    }
    files["index.md"] = render_index(indexed)
    validate(files)

    for filename, text in files.items():
        (OUT / filename).write_text(text, encoding="utf-8")

    table_count = sum(text.count("\n| ---") for text in files.values())
    link_count = sum(len(re.findall(r"\[[^\]]+\]\([^)]+\)", text)) for text in files.values())
    print(f"Generated {len(files)} Markdown files, {table_count} tables, and {link_count} links.")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode) from error
