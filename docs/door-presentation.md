# Door Presentation

*Status: renderer, local profile/geometry selection, unit coverage, and
OpenDoors/server integration coverage implemented, 2026-07-29*

The player door is a page-oriented, line-oriented terminal application. It
does not implement a cursor-addressed TUI. A page transition clears the
display, after which content is emitted as ordinary wrapped text. Enhanced
terminals receive colour and a larger character repertoire, but every
operation remains available through printable-key commands.

Presentation is entirely a client concern. The server sends structured
semantic data and never sends ECMA-48 controls, terminal glyphs, preformatted
screens, or layout decisions.

## Supported profiles

The door exposes three ordered output profiles:

| Profile | Text repertoire | ECMA-48 | Presentation |
|---|---|---|---|
| Plain | ISO 646 invariant repertoire | none | scrolling text |
| Colour | ISO 646 invariant repertoire | SGR colour and reset | scrolling text |
| CP437 colour | CP437 | SGR colour and reset | scrolling text |

The ISO 646 profiles use only the portable invariant text repertoire for
visible content. The colour profile adds rendition controls. The CP437
profile adds the DOS line, symbol, and accented-character repertoire while
retaining the same line-oriented interaction.

At a page transition, plain ISO 646 emits form feed (`0x0c`). Both enhanced
profiles emit SGR reset followed by `ESC [ 2 J` and `ESC [ H`. Clear-and-home
is the only cursor operation in the presentation contract: screens are never
assembled through coordinates, regions, or incremental redraw.

OpenDoors may convert CP437 output bytes to their UTF-8 equivalents for an
appropriate connection. That changes transport encoding, not the semantic
profile: it remains the CP437 repertoire rather than an unrestricted Unicode
interface.

## Semantic colour palette

The enhanced profiles use a high-contrast DOS/BBS palette inspired by the
multicolour data displays of *TradeWars 2002* and *Yankee Trader*. Colour is
semantic rather than an arbitrary rainbow: the same kind of value retains the
same colour across creation, trading, navigation, and combat screens.

| Role | ECMA-48 colour | Typical use |
|---|---|---|
| Heading | bright cyan | screen and section titles |
| Label | cyan | field names, units, and separators |
| Value | bright white | names and ordinary text values |
| Number | bright yellow | quantities, ratings, money, dates, and choices |
| Identifier | bright magenta | careers, phases, packages, and record IDs |
| Information | green | descriptions and informational state |
| Prompt | bright green | available commands and input prompts |
| Success | bright green | valid totals and completed/ready state |
| Warning/Error | bright red | invalid, negative, dangerous, or failed state |
| Normal/Muted | grey | prose and deliberately de-emphasized text |

The ISO 646 plain profile ignores roles and emits the exact same visible text
without rendition controls. No game meaning is conveyed by colour alone.

There is no TUI or general Unicode presentation profile. Apart from the
clear-and-home page boundary, the renderer must not depend on cursor
addressing, window save/restore, screen blocks, mouse input, function keys, or
terminal-specific character-width rules.

## Geometry

The supported minimum terminal is **40 columns by 24 rows**. The normal target
is **80 columns by 24 rows**. Larger terminals may reduce wrapping but must
not expose additional game actions unavailable at the minimum size.

The renderer owns wrapping and responsive record layout:

- no operation requires horizontal scrolling;
- values are wrapped or continued rather than silently truncated;
- wide tables become labeled records or a narrow essential-column view at 40
  columns;
- side-by-side information becomes sequential sections;
- decorative borders and indentation are minimized at narrow widths;
- long names, credit values, coordinates, errors, and legal notices remain
  readable; and
- ECMA-48 control bytes consume no display columns.

Menus remain line-oriented at both widths. Arrow keys may be accepted when
OpenDoors translates them, but every action has a printable-key equivalent.
For example, compact news review can show one item at a time and accept
printable commands for ignore, mark, next, and act as well as convenient arrow
aliases.

Height affects pagination rather than layout coordinates. The door may use
OpenDoors' reported screen length to place a continuation prompt, accounting
for any line reserved by the BBS or prompt. It must remain usable on an actual
40×24 terminal even when a legacy drop file reports only the usable page
length.

## OpenDoors boundary

OpenDoors already provides the required transport-facing facilities:

- plain and formatted output;
- basic colour and IBM-PC attributes;
- ANSI capability state and optional detection;
- key and extended-key translation;
- reported screen length;
- a `user_screenwidth` field when the selected drop-file format supplies it;
  and
- optional CP437-to-UTF-8 output conversion.

Screen width is not universally reliable. OpenDoors defaults it to 80, and its
manual states that only some legacy drop-file formats provide an actual
width. Its internal block, window, popup, and local-screen implementation is
also compiled around an 80×25 buffer.

The presentation profiles therefore use OpenDoors for connection handling,
input, text output, colour, and encoding assistance, but do not use its
fixed-coordinate window, block, popup, or screen-save APIs. The door maintains
an effective output width selected from:

1. an explicit session override (and, after drop-file startup, a persistent
   BBS default);
2. a credible width supplied by OpenDoors; or
3. the 80-column default.

The OpenDoors configuration directives are:

```text
CTConfig /secure/path/cepheus-trader.conf
CTProfile iso646|iso646-color|cp437-color
CTColumns 40..255
CTRows 24..255
```

`plain`, `color`/`colour`, and `cp437` are accepted aliases. Without an
explicit profile, an ANSI-capable OpenDoors session selects `iso646-color`
and another session selects `iso646`. CP437 is deliberately explicit because
ANSI capability alone does not attest the terminal character set. Without
geometry overrides, a credible OpenDoors value is used and then the 80×24
default. Changing terminal size during a door session is not detected; the
effective geometry remains stable for that session. OpenDoors owns command-line
and drop-file parsing; for example, local testing uses `-L -C doors.conf` and
may use OpenDoors' `-USERNAME` option to provide the account name.

## Content and safety

Every ordinary player-facing sentence is in-world. Screens name ships,
people, systems, worlds, facilities, dates, durations, instruments, and offices
as the captain would know them. Raw game seconds, database IDs presented as
implementation artifacts, placeholder stations, omniscient-server
explanations, and developer commentary are forbidden. Numeric warrant, claim,
offer, and task numbers are permitted when they are explicitly the in-world
instrument number. Unknown destinations are described as unlisted rather than
leaking an internal system identifier.

All server and player supplied text is sanitized before display. Control
characters, embedded ECMA-48 sequences, invalid encoding, and unintended line
breaks must not permit a player, ship, system, polity, or contract name to
change terminal state or forge interface text.

Text that cannot be represented by the selected profile is transliterated
where practical and otherwise replaced visibly. Rendering must be
deterministic so the same profile, width, and semantic view produce the same
output in tests.

## Test coverage

`client/tests/door_presentation_test.cpp` provides deterministic equivalent
assertions for:

- all three profiles at 40×24;
- all three profiles at 80×24;
- long and unrepresentable names;
- maximum supported credit, coordinate, date, and identifier values;
- dense ship, crew, market, contract, news, and error records;
- sanitization of control and ECMA-48 injection attempts;
- correct visible-width accounting around colour changes;
- the complete semantic-role palette and role-free plain output; and
- printable-key access to every action without arrow or function keys.

`server/tests/tls_interop.rs` separately runs the real door through OpenDoors
and the authenticated server at all three profiles. It verifies that plain
output uses form feed and contains no ECMA-48 sequence, enhanced output uses
reset/clear/home, local CP437-to-UTF-8 conversion preserves CP437 line glyphs,
and the effective-width override produces 40-column wrapping.
