# Cepheus Trader Licensing

Copyright (c) 2026 Cepheus Trader contributors

Cepheus Trader separates its software license from the license governing open
game rules and rule-bearing content. This separation is intentional and must
be preserved as the repository evolves.

## Original Software

Except where a file says otherwise, the original software implementation in
`server/`, `client/`, `protocol/`, `tools/`, and
`cepodnew-markdown/generate.py` is licensed under the MIT License below. This
includes server infrastructure, storage and transaction code, transport and
protocol implementation, terminal code, management utilities, generators,
build files, tests, and the original Cap'n Proto schemas.

This designation does not relicense third-party dependencies, generated code,
or Open Game Content under the MIT License.

### MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Open Game Content

All original game content authored for Cepheus Trader is Open Game Content
under the Open Game License version 1.0a. This includes:

- the upstream Open Game Content reproduced in `cepodnew-markdown/`, subject
  to its original Product Identity exclusions;
- original game mechanics, rules procedures, rules translations, tables, and
  mechanically significant rule definitions in `docs/` and
  `LLM_INSTRUCTIONS.md`; and
- every original name, description, mechanical field, role, tag, table,
  record, and other game-content element in the human-readable files under
  `catalog/`; and
- future original setting, character, organization, location, fiction, and
  other game content unless that material explicitly carries a different
  compatible open-content designation.

The complete Open Game License and consolidated Section 15 Copyright Notice
are in [OPEN_GAME_LICENSE.md](OPEN_GAME_LICENSE.md). Every source or binary
distribution containing Open Game Content must include that file. A title-only
source list is not a substitute for the complete Section 15 notice.

Software source does not become Open Game Content merely by being stored in
the same repository. Do not copy expressive Open Game Content into MIT-licensed
implementation files. Put rule text, tables, and other rule-bearing data in a
clearly identified Open Game Content file and have the software consume it.

## No Original Product Identity

Cepheus Trader designates no original game content as Product Identity. In
particular, **Cepheus Trader**, original ship class and variant names, setting
names, characters, organizations, locations, fiction, artwork, audio, and
trade dress are not reserved as Product Identity. When such material is
authored for and distributed as part of the game, it is Open Game Content
under the designation above unless it explicitly carries a different
compatible open-content designation.

Upstream Product Identity remains the property of its respective owners.
Nothing in this repository designates upstream Product Identity as Open Game
Content. In particular, Clement Sector proper names, organizations, locations,
characters, ship names, ship classes, artwork, and trade dress are not
licensed for reuse here by the Open Game License. Existing design notes that
refer to such names are research notes awaiting replacement, not a grant or
claim of ownership.

This no-reservation policy does not purport to waive or relicense third-party
Product Identity, third-party trademarks, or third-party software.

“Cepheus Engine” and “Samardan Press” are trademarks of Jason “Flynn” Kemp.
Cepheus Trader is not affiliated with Jason “Flynn” Kemp or Samardan Press.
Cepheus Trader is an Alternate Cepheus Engine Universe.

## Third-Party Software

Third-party software retains its own license. See
[THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for the current dependency
inventory and release obligations. No third-party code, generated code, or
linked library is designated as Open Game Content or relicensed under the MIT
License by this document.

## Required Maintenance

Every change that introduces a rule source or dependency must update the
applicable licensing records in the same change:

1. Update [docs/ogc-provenance.md](docs/ogc-provenance.md) when Open Game
   Content is copied, translated, or adapted from a new source.
2. Add every applicable exact attribution to
   [catalog/ogl-sources.toml](catalog/ogl-sources.toml), list the complete set
   of source IDs in each affected catalog entry, and regenerate the
   consolidated Section 15 declaration in
   [OPEN_GAME_LICENSE.md](OPEN_GAME_LICENSE.md).
3. Update [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) when a direct or
   transitive production dependency changes.
4. Do not add GPL, AGPL, SSPL, or similar strong-copyleft code to a linked
   production executable without a documented compatibility review.
5. Ensure every release package contains all applicable license texts and
   notices. The door's built-in license viewer supplements rather than
   replaces those files.
