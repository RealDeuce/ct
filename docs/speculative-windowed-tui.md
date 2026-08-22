# Speculative Windowed TUI UX Concept

*Status: exploratory UX notes. This work is not on the development roadmap and
has no implementation commitment.*

## Purpose and authority

This document records standard vocabulary and an initial UX concept for a
possible future terminal user interface.

The implemented and planned player interface remains the scrolling,
page-oriented door specified by
[`door-presentation.md`](door-presentation.md). All gameplay development,
acceptance testing, and balance work is performed through that interface. A
possible future TUI has no current requirements or acceptance criteria.

## UX concept

The concept is a **mode-based, windowed TUI**. The current application mode
selects a contextual workspace, within which the player interacts with
overlapping windows.

Candidate application modes include Docked, Voyage, and Combat.

Each contextual workspace may contain:

- a primary view representing the player's present operational situation;
- overlapping modeless windows for related information and actions;
- global windows, such as Ship, Crew, Messages, Tasks, or Known Universe,
  where they are meaningful in the current mode;
- a persistent window switcher at the top of the display; and
- a contextual command menu belonging to the active window.

A decision that must be completed or dismissed before other interaction can
continue may appear in a modal dialog above the workspace.

## Progressive terminal enhancement

The presentation concept can be understood as three progressively enhanced
tiers:

1. The scrolling text interface provides the universal gameplay and
   compatibility baseline.
2. A windowed TUI adds contextual workspaces, overlapping windows, keyboard
   focus, and contextual command menus.
3. A rich-terminal enhancement layer adds direct pointer interaction, pixel
   graphics, animation, and audio when the terminal exposes suitable
   capabilities.

Each tier supplements the one below it. Higher-tier presentation does not
introduce otherwise unavailable game information, decisions, or actions.

### Pointer interaction

Mouse input can provide direct activation of the window switcher, windows,
menus, controls, maps, and other visible targets. Keyboard interaction remains
available for the same tasks. Pointer focus, window activation, and keyboard
focus remain distinct UX concepts even when one mouse action changes all
three.

SyncTERM supports character-cell mouse reporting through SGR mouse mode 1006
and pixel-position reporting through mode 1016. Pixel coordinates could support
interaction with graphics whose meaningful regions do not align with the
character grid.

### Graphics and audio

Graphics and audio can reinforce significant subjects, transitions, and
events. Candidate uses include:

- displaying a ship image while examining a sale offer or vessel record;
- presenting Known Universe information as a graphical star map;
- revealing contact imagery as identification confidence improves;
- illustrating an important arrival, discovery, encounter, or combat result;
- providing brief cues for warnings, communications, transitions, and
  completed time-sensitive actions; and
- supplying restrained ambience appropriate to a contextual workspace.

Text continues to carry the identity, state, warning, and available decision.
Audio cues do not become the sole indication of an event, and graphical
controls retain keyboard-operable equivalents.

SyncTERM's CTerm extensions provide several possible facilities for this
layer:

- sixel pixel graphics;
- PPM and optional JPEG XL images supplied through the connection cache or as
  inline APC blobs;
- pixel buffers for copying and restoring areas of the display;
- positioned, clipped, masked, flipped, and integer-scaled image drawing;
- synthesized tones and decoded audio held in per-session patch slots; and
- concurrent audio channels with queues, loops, fades, crossfades, volume
  control, and completion notifications.

The authoritative command descriptions are in `src/conio/cterm.adoc` in the
Synchronet source tree.

### Capability adaptation

Rich-terminal presentation is selected by individual capabilities rather than
by assuming that every SyncTERM installation has the same facilities. CTerm
provides device-attribute and revision reports for general features, including
pixel operations and mouse support, plus specific queries for JPEG XL,
`libsndfile`, and supported audio container/subtype combinations.

The client can therefore adapt graphics, audio, and pointer behavior
independently. A missing or disabled capability removes that enhancement while
leaving the workspace, textual information, and commands intact.

Graphics and audio are most useful as deliberate points of emphasis. Routine
decoration, continuous animation, or persistent sound can compete with an
information-dense workspace and weaken the impact of exceptional events.

## Terminology

| Term | Meaning in this concept |
| --- | --- |
| Application mode | A state in which the available tasks and meaning of interaction differ, such as Docked or Combat. |
| Contextual workspace | The collection and arrangement of views appropriate to the current application mode. |
| View | A presentation of a game subject or activity. A view need not be a separate window. |
| Window | A bounded interface surface within a workspace. Windows may overlap. |
| Modeless window | A window that can remain open while the player activates another window. |
| Modal dialog | A blocking surface that must be resolved or dismissed before interaction with other windows resumes. |
| Active window | The window that currently receives window-level commands. |
| Keyboard focus | The control or menu item within the active window that currently receives input. |
| Z-order | The front-to-back stacking order of overlapping windows. |
| Window switcher | A persistent control used to find and activate open windows. |
| Contextual command menu | The actions offered by the active window. |
| Global window | A window potentially available in more than one application mode. |

## Conceptual interaction model

The following statements describe the idea without selecting keys, terminal
controls, or a rendering architecture:

- The current application mode determines the contextual workspace.
- Activating a modeless window brings it to the front of the Z-order.
- Cycling window activation is distinct from moving keyboard focus within the
  active window.
- The window switcher provides direct navigation among open windows.
- Ordinary commands are interpreted in the context of the active window.
- Each window presents the commands relevant to its subject or activity.
- Global windows may be available across multiple application modes.
- Notifications may request attention without taking activation or keyboard
  focus away from the player.
- A modal dialog temporarily owns interaction until it is resolved or
  dismissed.
- Changing application mode selects a different contextual workspace; the
  treatment of windows carried across that transition remains an open
  question.

## Fit with the current client/server boundary

The current CT-RPC boundary is broadly compatible with this concept. It sends
typed game state and commands rather than terminal screens, controls, glyphs,
or layout. Window management, activation, keyboard focus, Z-order, and
workspace arrangement would remain client concerns.

### Compatible protocol foundations

The protocol already provides several useful foundations for a retained,
multi-window client:

- A typed top-level player phase appears in the session hello and every
  response. An unsolicited phase-change event also carries the current travel
  status.
- Crew, ship, fleet, docked operations, services, markets, Tasks, finance,
  navigation, flight planning, messages, radio, encounters, combat, and
  career operations have separate semantic snapshots.
- Stable entity identifiers allow a client to retain selection and associate
  detail views with their subjects.
- Committed sequence numbers and revisions identify the state represented by
  responses. Expected revisions protect commands made from an older view.
- Stable command identifiers provide exactly-once transaction retry across a
  reconnect.
- Preview-and-commit exchanges, such as flight-plan review, naturally support
  decisions presented in modal dialogs.
- Combat snapshots carry participants, actors, allowed actions and reactions,
  default orders, deadlines, and revisions as structured data.
- CT-RPC text is independent of the repertoire selected by the current door
  renderer. ISO 646 and CP437 conversion occur at the presentation boundary,
  not in the game protocol.

The request and response surface in
[`ct_rpc.capnp`](../protocol/ct_rpc.capnp) already divides the game into
subjects that could correspond to windows. A future presentation would not
need window, focus, menu, or layout concepts added to the server protocol.

### Retained-view considerations

The current unsolicited event set covers phase changes, traffic, checkpoint
and encounter readiness, radio unread counts, session replacement, and server
shutdown. It does not announce changes to every snapshot that an open window
might retain, such as Crew, Ship, Tasks, Market, Finance, Messages, Fleet, or
Combat.

A client could work within the current protocol by refreshing a window when it
is activated. A continuously updated interface might instead benefit from
typed resource-invalidation events carrying a subject identifier and revision.
Such events would indicate that a view is stale; the client could then request
a fresh snapshot rather than receiving every changed snapshot unsolicited.

Combat is represented by a `CombatSnapshot` while the top-level player phase
remains `Encounter`. The current door discovers combat by requesting the
combat snapshot while resolving an encounter. A distinct activity
discriminator or combat-readiness event could make selection of a specialized
Combat workspace explicit, although the existing combat snapshot already
contains the state needed to populate one.

Every wire response carries committed-sequence, revision, and phase metadata.
The current C++ value types preserve that metadata for some snapshots but not
uniformly. A retained client state store would benefit from exposing it
consistently without requiring a wire-format change.

Action availability is structured in several important areas, including
docked services and combat-actor eligibility. Other menus combine typed state,
server-supplied explanatory text, and client-side presentation decisions. A
future client would need either shared presentation-independent view models or
its own equivalent menu policy while continuing to leave game-rule validation
on the server.

### Request dispatch

The wire envelope has request identifiers and permits events to arrive between
a request and its response. The current C++ protocol facade nevertheless waits
synchronously for one specific response and only defers intervening events.
It does not demultiplex several outstanding responses.

A windowed client could initially serialize protocol calls through a worker so
network waits do not block input or repainting. If concurrent observations
were useful, the client dispatcher could route responses by request identifier.
The server protocol would not necessarily need to change.

Although the wire schema contains a request-cancellation envelope, the current
player client and server do not use it as an established request path. A future
client should not assume that background observations are cancellable without
a separate protocol decision.

### Presentation and controller boundaries

The current `DoorPresentation` is a streaming text encoder, wrapper, and pager.
It tracks the current row and column but has no retained cell grid, window tree,
damage regions, or cursor-addressed compositor. It should remain the renderer
for the scrolling door rather than becoming an implicit window system.

The existing `door_main.cpp` screen functions combine protocol requests,
temporary state, rendering, input loops, menu policy, and mutations. Its event
handling converts unsolicited state directly into scrolling notices around the
active prompt. A retained interface would instead need shared client state,
view-specific state, event-driven invalidation, and a separate presentation
layer.

The build already keeps `ct-client-core` separate from
`ct-door-presentation` and OpenDoors. Protocol types, transport, display
formatting, and other presentation-independent facilities are therefore more
naturally reusable than the scrolling screen functions or renderer.

At the protocol level, a basic experiment could use the current contract with
serialized requests and refresh-on-activation. A more continuously updated
interface might motivate resource invalidation and an explicit
Encounter-versus-Combat activity signal. Most of the work implied by the UX
concept would remain in client state management and presentation rather than
in the game protocol.

## Open UX questions

The concept deliberately leaves these matters unresolved:

- whether the top-level window switcher shows only open windows, also launches
  common windows, or combines both roles;
- which windows are global and which belong to a single application mode;
- whether global windows retain their activation, position, selection, and
  navigation state across mode changes;
- whether more than one instance of a given window type may be open;
- how attention, unread state, urgent events, and time-sensitive decisions are
  represented;
- how much control a player has over window placement, size, and Z-order;
- how window menus, focus navigation, and application-wide commands coexist;
- how users enable, disable, or limit pointer, graphics, animation, and audio
  enhancements;
- how graphical regions participate in focus, activation, and accessibility;
- how cached media and graphical window contents behave across workspace and
  mode changes; and
- whether the overlapping-window model remains usable at every terminal size
  a future interface elects to support.

## Feasibility questions

No implementation investigation is presently scheduled. If the concept is
ever considered for planning, the terminal and OpenDoors boundary will require
fresh evaluation.

The currently vendored OpenDoors interface does not provide a general Unicode
terminal contract. Its optional conversion maps CP437 output bytes to their
UTF-8 encodings; it does not allow the application to emit arbitrary Unicode.
The available eight-bit-path indication likewise describes transport framing,
not a negotiated character encoding. General UTF-8 input, character-cell and
grapheme width, terminal capability discovery, geometry changes, and suitable
cursor-addressed output would therefore be investigation topics rather than
assumed facilities.

Those concerns do not affect current work because the scrolling door has its
own defined repertoire, geometry, rendering, and input contracts.
