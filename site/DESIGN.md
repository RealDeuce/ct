# Website Design Direction

## Purpose

The Cepheus Trader website is the player-facing companion to the BBS door. It
introduces the game, provides beginner help, and serves as a reference that a
captain can keep open during play. It should feel like another way of accessing
the game's universe, not a separate marketing site laid over it.

This document records the intended direction. It is a guide for extending or
revising the site, not a requirement to preserve every detail of the current
CSS.

## Core idea

Cepheus Trader is based on rules first designed in the mid-to-late 1970s. Its
far future is therefore the future imagined from that period, including the
assumptions, industrial forms, printed matter, and visual language that carried
into 1980s science fiction.

The website should look like a useful artifact from that imagined future. Its
reference pages are a captain's library, operations manual, port briefing, or
dispatch service. The presentation may suggest that the player is consulting
the same institutional material that exists in the setting.

Three priorities govern every page, in order:

1. **Useful during play.** Information must be quick to find, legible for a
   long session, and usable on desktop and mobile screens.
2. **Believable in-world.** Navigation, labels, illustrations, and incidental
   copy should support the fiction whenever that does not obscure meaning.
3. **Period-future character.** The site should evoke 1970s and 1980s
   science-fiction publishing and industrial design, not a contemporary web
   product wearing a retro filter.

## The visual world

The desired character combines two related kinds of artifact:

- **Printed matter:** a well-used starship manual, trade circular, government
  dossier, navigation chart, or paperback-era technical illustration.
- **Command equipment:** a durable shipboard or port terminal with labelled
  controls, plotted vectors, status lamps, instrument colors, and information
  arranged for an operator rather than a consumer.

The result should feel practical, slightly institutional, and lived in. Space
travel is work. Ships are expensive machines; information arrives through
specific channels; paperwork, authority, and delayed messages matter. The site
can be dramatic, but it should not make the setting frictionless or magical.

### Reference points

Use broad period traditions rather than imitating a particular franchise,
artist, book cover, game edition, logo, or trade dress:

- painted and airbrushed science-fiction covers;
- restrained technical manuals and aerospace diagrams;
- dense but orderly role-playing reference books;
- CRT displays, plotter output, paper forms, and physical control panels;
- blocky industrial equipment with visible seams, labels, and service access;
- optimistic color set against the wear and seriousness of working machinery.

These are directional references only. New work should be original and should
not reproduce protected imagery or the recognizable visual identity of another
science-fiction property.

## Color, type, and texture

The current palette is a sound expression of the direction:

- charcoal and near-black for space, equipment, and dark reading surfaces;
- warm off-white for paper, plotting stock, and primary text;
- burnt orange and amber for action, warning, emphasis, and illumination;
- muted teal for navigation, data, status, and institutional accents;
- red only for genuine danger, destructive action, or failure.

Colors should look printable or produced by equipment. Avoid candy-bright
rainbows, pervasive neon, glossy gradients, and the blue-purple-pink shorthand
often used for modern "retro" or synthwave design. Accent colors must retain
their meaning and must never be the only way information is communicated.

The type system likewise has three jobs:

- a condensed, assertive sans serif for titles, placards, and navigation;
- a comfortable serif for sustained reading and manual prose;
- a monospace face for document codes, coordinates, key commands, status, and
  machine-produced metadata.

Uppercase, tracking, and condensed type add character in short labels. They
should not be used for paragraphs or other long reading. Body text must remain
calm and generous enough to consult during a play session.

Texture should be subtle: paper grain, ink, a faint screen or plotting grid,
and slight tonal variation are appropriate. Distress, blur, glow, scan lines,
chromatic aberration, and animation must not reduce legibility. The site evokes
old media without pretending that the player's display is broken.

## Composition and imagery

Prefer strong editorial composition over a conventional stack of rounded web
cards. Useful devices include:

- asymmetric page grids and large periodical-style headings;
- thin rules, keyed sections, marginal notes, and document identifiers;
- tables that resemble schedules, manifests, or equipment registers;
- orbital plots, vectors, deck-plan geometry, technical silhouettes, and
  annotated cutaways;
- occasional paper-colored sections that feel inserted into a dark console or
  opened on a desk;
- squared or clipped corners and solid color fields rather than glass effects,
  floating pills, and soft shadows.

Illustration should show the material setting: ships with scale and structure,
ports, cargo handling, crew at work, worn machinery, charts, and distant worlds.
When painterly art is used, favor the compositional confidence and physical
media of 1970s/1980s science-fiction art. When diagrams are used, favor
clarity and plausible annotation over decorative complexity.

Ship-catalog illustration follows the detailed family, shipyard, component,
scale, and production rules in
[`SHIP-ART-GUIDE.md`](SHIP-ART-GUIDE.md).

Do not default to sleek contemporary spacecraft, weightless holograms, generic
cyberpunk cities, 1950s pulp rockets, steampunk ornament, or militaristic
franchise mimicry. The world may contain advanced technology, but its visual
language is tactile, engineered, and understandable to its operators.

## In-world presentation

Use in-world framing wherever it improves immersion without making the site
harder to understand. Existing phrases such as **Captain's library**,
**Incoming dispatch**, **First watch**, and **Transmission continues while you
are away** establish the right voice.

Good in-world presentation includes:

- describing help as material issued to or carried by a captain;
- giving sections short document codes, channel names, dates, or provenance;
- presenting navigation and status as operational language;
- using examples that name plausible ports, ships, offices, and messages;
- treating delays, local knowledge, law, ownership, and physical custody as
  ordinary facts of the world.

The fiction must not conceal the interface. A new player should still recognize
links, search, navigation, warnings, version information, and the difference
between game rules and atmosphere. Prefer a clear label with an in-world
subtitle over an opaque invented term. For example, **Beginner Help — Captain
Orientation Channel** is better than a channel code with no explanation.

Some information cannot honestly be in-world. Licensing, source attribution,
privacy, browser requirements, repository links, bug reporting, and service
status should be plain and explicit. Such material can share the visual system,
but it should not be disguised as fiction.

## Writing voice

The site addresses the player as a capable new captain. Its voice is concise,
observant, and concrete. It respects the danger and cost of the setting without
becoming grim or pompous.

- Prefer specific nouns and active verbs: berth, warrant, manifest, plot,
  inspect, accept, depart.
- Let the world emerge through useful detail rather than long invented lore.
- Use institutional language where an actual institution is speaking, and a
  direct instructional voice where the player needs help.
- Preserve uncertainty when the game does: a report has a source and an age;
  it is not universal truth.
- Do not turn ordinary controls into florid role-play prose.
- Avoid present-day startup, social-media, and software-as-a-service language.

Atmospheric headings can carry more personality than instructions. The player
should never have to decode flavor text to learn what a key does or what a rule
means.

## Information architecture for play

The site is a working reference, not only an introduction. Changes should
optimize for a player who has left the door open in another window and needs an
answer quickly.

- Keep global navigation small, stable, and predictable.
- Provide search for help and, when the reference grows enough to warrant it,
  for the full manual.
- Preserve durable headings and fragment links so players can share an exact
  answer.
- Put the answer before atmospheric elaboration on reference pages.
- Keep key commands, requirements, costs, risks, and consequences visually
  scannable.
- Make tables usable on narrow screens without destroying their relationships.
- Clearly identify whether text is live game help, the player manual, an
  example, or setting flavor.
- Avoid hiding essential information behind animation, hover, carousels, or
  elaborate simulated controls.

The landing page may be cinematic and editorial. Reference and help pages must
be quieter, denser, and optimized for repeated use.

## Accessibility and resilience

The period character must be achieved through composition and art direction,
not through barriers to use.

- Use semantic HTML, logical heading order, visible keyboard focus, and useful
  alternative text.
- Meet WCAG AA contrast for text and controls.
- Support keyboard-only navigation, zoom, reduced motion, and narrow screens.
- Keep touch targets large enough even when they look like labelled controls.
- Maintain a useful print style for manuals and reference pages.
- Do not require custom fonts, JavaScript, animation, audio, or large images to
  reach the core documentation.
- Favor fast pages that remain practical over slow BBS connections and modest
  hardware.

Progressive enhancement is part of the fiction's credibility: a captain's
manual should still work when a decorative system is unavailable.

## Current implementation

`assets/site.css` already establishes the dark equipment/paper palette, the
condensed-serif-monospace type roles, technical grid, clipped wordmark, orbital
plot, editorial sections, responsive layouts, reduced-motion behavior, and
print treatment. `build.py` establishes the captain's-library and dispatch
voice.

Those choices are a foundation, not a closed design. Future work may refine the
layout, typography, art, or component system as long as it serves the priorities
and period-future direction above.

When reviewing a visual change, ask:

1. Can a player find and read what they need during play?
2. Does this feel like material from the game's world?
3. Does it express the far future as imagined in the 1970s and 1980s?
4. Is the result original rather than an imitation of a known property?
5. Does it remain accessible, fast, and robust?
