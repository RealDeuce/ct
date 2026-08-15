# Cepheus Trader client

This directory contains the C++20 player and management clients for Cepheus
Trader. Players can start with the [player guide](../docs/player-guide.md),
installed as `PLAYER-GUIDE.md` in packaged client kits. BBS operators should
use the [sysop guide](../docs/sysop-guide.md), installed as `SYSOP-GUIDE.md`.

## Programs and libraries

- `cepheus-trader-door` is the OpenDoors player door.
- `cepheus-trader-sysop` manages one BBS's configuration, identities, and
  player access.
- `cepheus-trader-admin` manages the authoritative server through its
  loopback-only administrator listener.
- `cepheus-trader-client` is a headless protocol and test exerciser.

All four executables use `cepheus-trader-client-core`, a shared TLS and
cryptography transport library. Its DLL boundary is a narrow C interface:
opaque handles, fixed-size values, byte buffers, and status codes. C++
exceptions and standard-library objects stay on the side that created them.
Failures cross as a structured snapshot containing a stable category, an
optional native error number, and a caller-copied, untruncated UTF-8 message.
Cap'n Proto and common protocol handling are linked statically into each
program. The door additionally links OpenDoors as a private static library.
Game rules and persistent state remain on the Rust server.

The door supports ISO 646 plain text, ISO 646 with ECMA-48 colour, and CP437
with ECMA-48 colour. A 40x24 terminal is the supported minimum and 80x24 is the
normal target. OpenDoors supplies connection handling, input, output, colour,
encoding assistance, command-line parsing, and drop-file parsing; the project
owns responsive presentation and identity mapping. See the
[door presentation design](../docs/door-presentation.md).

## Build and test

The client requires CMake 3.20 or newer, a C++20 compiler, Python 3,
pkg-config, and a supported build tool such as Ninja or GNU Make:

```console
cmake -S client -B client/build -G Ninja
cmake --build client/build
ctest --test-dir client/build --output-on-failure
```

CMake prefers suitable installed Botan 3 and Cap'n Proto packages. Otherwise
it downloads the pinned official Botan and Cap'n Proto source releases,
verifies their SHA-256 digests, and builds them inside the build tree. Use
`CT_USE_SYSTEM_BOTAN=OFF` or `CT_USE_SYSTEM_CAPNPROTO=OFF` to exercise the
fallback builds.

Automation may provide completed dependency installations through
`CT_BOTAN_ROOT`, `CT_CAPNPROTO_ROOT`, and `CT_OPENDOORS_ROOT`. These prefixes
take precedence over system discovery and in-tree builds. With a Makefile
generator, the Botan build inherits the parent GNU Make jobserver; other
generators use the processors detected by CMake unless
`CT_BOTAN_BUILD_JOBS=N` sets an explicit limit.

The Rust TLS interoperability test launches the administrator, sysop,
headless, and OpenDoors clients against the real GnuTLS server. It covers
credential bootstrap, configuration, authenticated hello exchange, player
creation, presentation profiles, reconnects, identity management, and sysop
moderation.

## Packaging

Release builders set `CT_PORTABLE_CLIENT=ON`, build pinned Botan and Cap'n
Proto, link OpenDoors statically, and use static GNU C and C++ runtimes on
supported GNU targets. CPack includes only the player door, sysop utility,
their shared TLS transport, the player and sysop guides, configuration
example, source correspondence, and license notices. The administrator and
headless exerciser are development and server-operation programs, not BBS
client-kit contents.

The packaged binaries must have an exact tagged source release and satisfy the
obligations in [THIRD_PARTY_LICENSES.md](../THIRD_PARTY_LICENSES.md). OpenDoors
source and its relinking build files are vendored beneath
`third_party/opendoors`.
