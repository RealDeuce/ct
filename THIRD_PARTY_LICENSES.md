# Third-Party Software Licenses

This file records the production dependency license boundary. It is an
inventory and release checklist, not a replacement for the copyright notices
and complete license texts that must accompany a binary distribution.

## Policy

- Permissive dependencies such as MIT, ISC, BSD, Apache-2.0, Unlicense, and
  Unicode are acceptable when their notice requirements are preserved.
- LGPL libraries are acceptable only with an explicit compliance plan.
  Cepheus Trader's OpenDoors plan is to ship the complete corresponding
  source, local modifications, build files, and license in the exact tagged
  source release beside each statically linked client package, allowing a
  recipient to rebuild and relink with a modified OpenDoors library.
- GPL, AGPL, SSPL, and comparable strong-copyleft dependencies must not be
  linked into a production executable without a documented license
  compatibility review. A genuinely separate program communicating over a
  protocol is evaluated separately.
- Build tools do not license their output merely by running during the build,
  but copied runtime code and compiler runtime libraries still require review.
- Before every release, regenerate the complete transitive inventory for every
  supported target. A package-manager summary is not a substitute for the
  dependency's actual license files.

## Current Direct Production Dependencies

| Component | Used by | License | Link/boundary |
| --- | --- | --- | --- |
| Cap'n Proto | C++ clients and Rust server | MIT | Installed or pinned CMake-built C++ library; Rust crate |
| Botan 3 | C++ clients | BSD-2-Clause | System library for development or pinned 3.12.0 static source build |
| OpenDoors | Player door | LGPL-2.0-or-later | Vendored source, statically linked in official client packages |
| GnuTLS core library | Rust server | LGPL-2.1-or-later | Shared library |
| `capnp` | Rust server | MIT | Rust crate |
| `getrandom` | Rust server | MIT OR Apache-2.0 | Rust crate |
| `heed` | Rust server | MIT | Rust crate |
| `lmdb-master-sys` | Rust server through `heed` | Apache-2.0 | Rust/native code |
| `thiserror` | Rust server | MIT OR Apache-2.0 | Rust crate |
| `tokio` | Rust server | MIT | Rust crate |

The installed GnuTLS package also contains GPL utilities. Cepheus Trader links
the LGPL GnuTLS core library and does not copy or link those utilities.

The client builds the audited source under `client/third_party/opendoors/` as
a static library. Its provenance and copied file list are recorded in that
directory's `README.md`. Every binary release must publish the exact tagged
source archive beside the binaries and include the OpenDoors source, local
modifications, build files, copyright notices, and LGPL text needed to rebuild
and relink the client with a modified library.

Ordinary developer builds prefer a suitable installed Botan. Reproducible
portable builds set `CT_USE_SYSTEM_BOTAN=OFF`, which downloads the official
Botan 3.12.0 source archive and verifies SHA-256
`5370f98dc15f8c222ee1ce52cd61c8756a53be0dc57cc4c1b0714d5a09ad74fb`
before building its static library. The repository carries the Botan and
Cap'n Proto license notices used by client packages.

## Current Rust Transitive Inventory

`cargo metadata` currently reports only permissive, Unicode, or selectable
permissive licenses in the normal production graph. The precise graph is
locked by `server/Cargo.lock`; it must be re-audited whenever that file
changes. Development-only crates must also be reviewed if they are ever
shipped in a release artifact.

Useful audit commands are:

```console
cd server
cargo tree --edges normal
cargo metadata --format-version 1
```

Do not infer a crate's effective license solely from a package manager's
abbreviated expression. Preserve the selected upstream license and notices in
the release.
