# Cepheus Trader

[![CI](https://github.com/RealDeuce/ct/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/RealDeuce/ct/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/RealDeuce/ct?sort=semver)](https://github.com/RealDeuce/ct/releases/latest)
[![Software: MIT](https://img.shields.io/badge/software-MIT-blue.svg)](LICENSE.md)
[![Game content: OGL 1.0a](https://img.shields.io/badge/game_content-OGL_1.0a-blue.svg)](OPEN_GAME_LICENSE.md)

Cepheus Trader is a multiplayer BBS door game inspired by *TradeWars 2002*
and *Yankee Trader*. It combines trading, ship operation, interstellar
mail, exploration, naval service, privateering, piracy, and space combat in a
persistent 3D universe based on the Cepheus Engine rules.

The project is in active alpha development at version **0.7.15**. Milestones 0
through 6 are complete; Milestone 7 is validating multi-BBS play and field
operations. See the [roadmap](ROADMAP.md) for the authoritative status and
acceptance boundaries.

New and returning captains can use the
[player documentation site](https://realdeuce.github.io/ct/) for the game
introduction, complete player reference, and searchable Beginner Help.

Cepheus Trader is an Alternate Cepheus Engine Universe. It is not affiliated
with Jason “Flynn” Kemp or Samardan Press.

## Architecture

One authoritative Rust server owns all game rules and persistent state. BBS
players connect through a C++20 OpenDoors client over a TLS 1.3 external-PSK
protocol using Cap'n Proto messages. Separate clients provide server
administration and per-BBS sysop management.

The repository is self-contained: the OpenDoors sources used by the player
door are vendored under `client/third_party/opendoors`. Building Cepheus Trader
does not require a Synchronet source checkout or xpdev.

## Repository layout

- [`server/`](server/) — authoritative Rust server and simulation
- [`client/`](client/) — OpenDoors player door, sysop/admin tools, and protocol
  exerciser
- [`protocol/`](protocol/) — shared Cap'n Proto schemas
- [`catalog/`](catalog/) — game data, ship designs, and attribution records
- [`docs/`](docs/) — game, protocol, storage, and operational specifications
- [`cepodnew-markdown/`](cepodnew-markdown/) — generated local Cepheus Engine
  rules reference

The high-level game design and previously README-hosted implementation detail
are preserved in the [game design overview](docs/game-design.md).

## Build and test

The server requires stable Rust, Cap'n Proto, and GnuTLS 3.8.11 or newer:

```console
cargo build --manifest-path server/Cargo.toml
cargo test --manifest-path server/Cargo.toml
```

The client requires CMake 3.20 or newer, a C++20 compiler, Ninja, pkg-config,
and Python 3. It uses installed Botan 3 and Cap'n Proto packages when suitable;
otherwise CMake downloads and verifies the pinned source releases.

```console
cmake -S client -B build/client -G Ninja
cmake --build build/client
ctest --test-dir build/client --output-on-failure
```

Run the repository and catalog checks with:

```console
python3 tools/check_repository.py
python3 -m unittest discover -s tools -p 'test_*.py'
```

## Running the server

The server listens on `localhost:7323` for players, `localhost:7324` for
administrators, `localhost:7325` for BBS sysops, and `localhost:7326` for
League Coordinators by default. `localhost` is resolved at startup and every
supported IPv4 and IPv6 result is bound.

The listener options are repeatable and also bind every address returned for
a hostname:

```console
cepheus-trader-server \
  --listen 0.0.0.0:7323 --listen '[::]:7323' \
  --sysop-listen 0.0.0.0:7325 --sysop-listen '[::]:7325' \
  --league-listen 0.0.0.0:7326 --league-listen '[::]:7326'
```

The administrator listener remains restricted to loopback addresses. Explicit
listener values are strict: startup fails if any requested address cannot be
resolved or bound.

See the [sysop guide](docs/sysop-guide.md) for BBS installation,
configuration, credential bootstrap, and door setup. Client development is
covered by the [client documentation](client/README.md), League operation by
the [League Coordinator guide](docs/league-coordinator.md), and portable
packages by the [release process](docs/release-process.md). Optional Web Push
deployment is covered by the [browser-alert guide](docs/browser-alerts.md).

## Documentation

- [Player documentation site](https://realdeuce.github.io/ct/)
- [Development roadmap](ROADMAP.md)
- [Player guide](docs/player-guide.md)
- [Guided First Watch design](docs/guided-first-watch.md)
- [BBS sysop installation and operations](docs/sysop-guide.md)
- [League Coordinator administration](docs/league-coordinator.md)
- [Operator-hosted interactive universe atlas](docs/universe-atlas.md)
- [Game design and implementation overview](docs/game-design.md)
- [Milestone 7 field-alpha audit](docs/milestone-7-audit.md)
- [Protocol and storage model](docs/rpc-and-storage-schema.md)
- [Door presentation and OpenDoors boundaries](docs/door-presentation.md)
- [Speculative windowed TUI UX concept](docs/speculative-windowed-tui.md)
- [Implementation guidance](LLM_INSTRUCTIONS.md)

## Licensing

Original software is available under the MIT License. Original game content
and rule-bearing material are Open Game Content under the Open Game License
1.0a. Third-party software retains its own licenses.

See [LICENSE.md](LICENSE.md), [OPEN_GAME_LICENSE.md](OPEN_GAME_LICENSE.md),
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md), and the
[OGC provenance record](docs/ogc-provenance.md) for the complete terms and
attributions.
