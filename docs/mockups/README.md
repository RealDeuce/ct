# Speculative TUI mockups

*Status: exploratory artifacts. These mockups are not on the development
roadmap and do not define implementation requirements.*

The paired mockups show the same Docked-mode ship-market state and the same
available actions:

- [Basic character-cell TUI](speculative-tui-ship-market-basic.svg) uses a
  keyboard-operated window switcher, overlapping windows, textual ship data,
  and a contextual command menu.
- [Capability-enhanced TUI](speculative-tui-ship-market-enhanced.svg) retains
  that structure and adds direct pointer targets, existing ship artwork,
  graphical status accents, and graphics/audio capability indicators.

The enhanced view is progressive presentation, not a different gameplay
interface. Every identity, fact, warning, and action shown there has a textual
counterpart in the basic view. The offer price, seller, and inspection state
are illustrative UI content rather than defined game data; the Hermes design
facts come from `catalog/ships/ship-1.toml`.

The enhanced mockup references the existing Hermes image at
`site/assets/ships/ship-001-hermes.webp` rather than duplicating the asset. It
also contains a simplified vector fallback for SVG viewers that block linked
resources.
