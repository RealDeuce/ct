# Release process

Cepheus Trader uses one product version across Cargo, CMake, executable
version output, archive names, and the GitHub release. The initial product
version is `0.7.0`; project-owned wire, storage, record-codec, and generator
compatibility counters independently begin at 1.

The ordinary GitHub Actions workflow validates repository hygiene and
licenses, runs the Python/catalog suite, Rust formatting/lint/tests, and runs
the native CMake/CTest suite on Ubuntu. Its client package matrix uses GitHub
hosted runners to build only `cepheus-trader-door` and
`cepheus-trader-sysop` for Linux amd64/arm64, macOS arm64/x86-64, and Windows
x86-64/x86. Linux packages are built once per architecture on Ubuntu 22.04,
then the same archives are exercised on Ubuntu 22.04, 24.04, and 26.04.
Each client kit also contains the platform's `cepheus-trader-client-core`
shared library, which owns the Botan-backed TLS transport behind a stable C
ABI. The common protocol and Cap'n Proto/KJ implementation are linked into
the executables.

Botan, Cap'n Proto/KJ, and OpenDoors each have an independent cache per
platform ABI and toolchain. They are rebuilt only on a cold cache or when the
corresponding pinned source/build recipe changes. Normal client jobs build
the code owned by this repository, including the shared TLS transport and
statically linked protocol code, and consume those cached dependency
libraries.

There is no FreeBSD CI coverage. Field-alpha load and the full capacity
fixture are manual benchmarks with retained reports.

Version tags run the same hosted matrix and publish its verified client
artifacts to a GitHub release after all required CI jobs pass.

For an alpha release:

1. make the default-branch pipeline green;
2. confirm `python3 tools/check_repository.py` succeeds on the tag commit;
3. create the protected annotated tag `v<version>` (for example, `v0.7.6`);
4. let each native runner configure with `CT_PORTABLE_CLIENT=ON` and an exact
   GitHub release URL;
5. unpack and inspect every archive, run native `--version` probes, and inspect
   dynamic dependencies; and
6. keep GitHub's exact tagged source archive available beside all binaries.

Every client archive contains `cepheus-trader-door`, `cepheus-trader-sysop`,
their platform-specific `cepheus-trader-client-core` shared library, the
player and sysop guides, example configuration and installation notes, all
project and dependency notices, and `SOURCE-RELEASE.txt`. The tagged source is
the OpenDoors corresponding-source distribution: it includes the vendored
library, local modifications, schemas, and build files needed to rebuild and
relink. Do not publish a binary if its source-release URL is empty.

The first alpha packages may be unsigned. Windows signing and macOS
signing/notarization become separate release-only jobs after credentials are
provisioned as protected GitHub Actions secrets. Signing material never
belongs in the repository, ordinary job logs, command-line arguments, or
environment variables exposed to unprotected jobs.
