# Release process

Cepheus Trader uses one product version across Cargo, CMake, executable
version output, archive names, and the GitHub release. The initial product
version is `0.7.0`; project-owned wire, storage, record-codec, and generator
compatibility counters independently begin at 1.

The ordinary GitHub Actions workflow validates repository hygiene and
licenses, runs the Python/catalog suite, Rust formatting/lint/tests, native
CMake/CTest, the pinned Cap'n Proto and Botan build path, and a native
portable-package audit. Field-alpha load and the full capacity fixture are
manual benchmarks with retained reports.

Tag workflows use explicitly tagged self-hosted native runners. The release
matrix covers Linux amd64/arm64, FreeBSD 14 amd64, macOS arm64/x86-64, and
Windows x86-64/x86. Runner tag names in `.github/workflows/release.yml` are
the desired contract and may be adjusted when the project's actual runner
fleet is registered. A missing runner leaves only its platform job pending;
it does not justify building an incompatible package on another operating
system.

Release runners provide CMake, Ninja, Python 3, a C/C++ toolchain, and archive
tools. MinGW runners additionally provide the selected cross compiler and
make program plus host-native `capnp` and `capnpc-c++` schema generators.
FreeBSD and benchmark runners use the labels recorded in their workflows.

For an alpha release:

1. make the default-branch pipeline green;
2. confirm `python3 tools/check_repository.py` succeeds on the tag commit;
3. create the protected annotated tag, initially `v0.7.0-alpha.1`;
4. let each native runner configure with `CT_PORTABLE_CLIENT=ON` and an exact
   GitHub release URL;
5. unpack and inspect every archive, run native `--version` probes, inspect
   dynamic dependencies, generate SHA-256 checksums, and retain separate debug
   symbols where the platform produces them; and
6. keep GitHub's exact tagged source archive available beside all binaries.

Every client archive contains only `cepheus-trader-door` and
`cepheus-trader-sysop`, the example configuration and installation notes, all
project and dependency notices, and `SOURCE-RELEASE.txt`. The tagged source is
the OpenDoors corresponding-source distribution: it includes the vendored
library, local modifications, schemas, and build files needed to rebuild and
relink. Do not publish a binary if its source-release URL is empty.

The first alpha packages may be unsigned. Windows signing and macOS
signing/notarization become separate release-only jobs after credentials are
provisioned as protected GitHub Actions secrets. Signing material never
belongs in the repository, ordinary job logs, command-line arguments, or
environment variables exposed to unprotected jobs.
