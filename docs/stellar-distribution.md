# Stellar Distribution Model

*Status: version 1 density and arm geometry implemented, 2026-07-28;
coverage oracle, materialization bits, and bulk capacity sampling implemented;
ordinary Jump-arrival sampling pending*

This model supplies a continuous expected density of stellar components at
any Cepheus Trader coordinate. It does not pre-generate the Galaxy and does
not introduce a master random seed. Frontier generation samples this
intensity with the operating-system cryptographic random source and persists
the realized systems and surveyed empty volume.

The implementation is in `server/src/universe.rs` and is identified by
`STELLAR_DISTRIBUTION_VERSION = 1`. A stored frontier-region record must carry
the distribution version under which it was resolved. Changing the model
must not silently reroll an already surveyed region.

## Coordinate transform

Game coordinates remain Earth-centered parsecs:

- positive X is coreward;
- positive Y is spinward;
- positive Z is Galactic north.

The density calculation converts those values to Galactocentric cylindrical
coordinates:

```text
Xg = R0 - X
Yg = Y
R  = sqrt(Xg² + Yg²)
θ  = atan2(Yg, Xg)
Zg = Z + Zsun
```

`θ` increases spinward. At the game origin, `R = R0` and `Zg = Zsun`.
This is a calculation transform, not a second player-facing map system.

Version 1 uses:

| Parameter | Value | Purpose |
| --- | ---: | --- |
| Solar Galactocentric radius, `R0` | 8,178 pc | Places Earth relative to the Galactic center |
| Solar height, `Zsun` | +20.8 pc | Places Earth above the Galactic midplane |
| Local stellar-component density | 0.0906 pc⁻³ | Stars plus brown dwarfs at the game origin |
| Radial scale length | 2,200 pc | Smooth exponential coreward/rimward gradient |
| Thin-disk scale height | 300 pc | Dominant vertical component |
| Thick-disk scale height | 900 pc | Extended vertical tail |
| Thick-disk midplane fraction | 0.06 | Relative weight of the extended component |

The local density is the sum of the CNS5 stellar density, 0.0799 pc⁻³, and
brown-dwarf density, 0.0107 pc⁻³. The initial 35-component Federation remains
an explicit observed local realization and is not used to renormalize the
large-scale distribution.

At an otherwise unresolved Solar-neighborhood point, this intensity gives an
unconditioned mean of about 0.38 components within one parsec and 3.04 within
two parsecs. Actual Poisson counts vary, and existing fixed systems condition
the result in already surveyed space. The model therefore reproduces the
basic gameplay shape already observed in the initial catalog: Jump-1 links
are uncommon, while Jump-2 normally finds several candidates without
guaranteeing a connected route everywhere.

The adopted Galactic-center distance comes from the GRAVITY geometric
measurement. The radial scale is a rounded stellar-disk approximation near
the 2.15 kpc mass-weighted measurement. Real stellar populations have a
continuous range of scale lengths and heights; the two vertical components
are a compact game model, not a claim that the Milky Way has two discrete
populations.

## Repeated spiral arms and direction

Version 1 has four identical trailing logarithmic arms. Treating the arms as
copies means they share pitch, width, and density contrast while differing
only by a quarter-turn phase:

```text
φk = π/4 + k(2π/4), for k = 0..3

ln(R/R0) = -(θ - φk) tan(p)
```

The pitch `p` is 10 degrees. The sign makes an arm run inward/coreward as it
runs spinward, producing a trailing spiral. The `π/4` offset places Earth
between two generic major arms at the Solar circle instead of asserting that
the local Orion spur is a fifth equal arm.

Arm direction is therefore not a fixed spinward stripe. At every position the
model derives the local unit tangent of the nearest arm. In Galactocentric
radial/azimuthal components, its spinward-pointing tangent is:

```text
t = -sin(p) eR + cos(p) eθ
```

The server converts that vector back to coreward/spinward/north components.
Its coreward and spinward values rotate continuously with Galactic azimuth.
On the Sun--Galactic-center line, the local spinward tangent is approximately
`[+0.174 coreward, +0.985 spinward, 0 north]`; the reverse direction has all
signs reversed. This is only a local example, not a fixed arm direction.

For density, define the logarithmic phase coordinate and wrapped separation:

```text
q  = θ + ln(R/R0) / tan(p)
δk = wrap(q - φk)
dk = R sin(p) δk
```

`dk` is the signed local normal offset from arm `k`. It is exact on the
centerline and is the useful perpendicular-distance approximation within the
arm-width region. Version 1 uses a 350 pc Gaussian width and a 0.35 peak
overdensity for every arm:

```text
A(R,θ) = 1 + 0.35 Σ exp(-dk² / (2 × 350²))
```

The four-arm pitch is observationally motivated; the common width, common
contrast, and equal-arm assumption are deliberate game simplifications.

## Complete density

The vertical factor is:

```text
V(Zg) =
    0.94 exp(-|Zg| / 300)
  + 0.06 exp(-|Zg| / 900)
```

The final component intensity is:

```text
ρ(X,Y,Z) =
  ρsun
  exp((R0 - R) / 2200)
  V(Zg) / V(Zsun)
  A(R,θ) / A(R0,0)
```

The normalization terms make the result exactly 0.0906 components pc⁻³ at
Earth. Coreward/rimward and spinward/trailing are coupled through `R`, `θ`,
and the curved arm phase; they are not three independent one-dimensional
random distributions. North/south supplies the separate vertical falloff.

## Frontier materialization

The persistent quarter-parsec coverage lattice and coordinate oracle are
specified in
[`observed-volume.md`](observed-volume.md). The density function is the
intensity for an inhomogeneous Poisson process. For each set of missing cells
returned by that oracle, the materializer should:

1. determine a conservative maximum density over each chunk's missing cells;
2. draw a homogeneous Poisson candidate count from that maximum;
3. choose candidate missing cells and positions uniformly and retain each with
   probability
   `ρ(position) / ρmaximum`;
4. seed a cryptographic stream once from fresh 256-bit OS-CSPRNG entropy and
   assign each retained component the next prospective system seed;
5. atomically persist the generated components and the resolved and empty
   cell bits.

The implemented settlement-capacity operation applies the same point-process
semantics to one bounded Sol-centered sphere. It uses an analytic upper bound
over the whole sphere, exponential inter-arrival sampling for the homogeneous
Poisson count, uniform spherical positions, density thinning, and rejection
of cells already resolved by the CNS5 or BBS-conditioned layers. Its fixed
operation seed exists only to make the capacity fixture reproducible. Normal
frontier operations must take fresh OS-CSPRNG entropy and retain the
chunk-scoped compare-and-set behavior above.

Overlapping requests carry a coverage revision and must resolve only the
still-unresolved cells inside the authoritative transaction. Decomposing a new
frontier in a different order may produce a different realization, which is
consistent with the existing visit-order-dependent universe decision. Once
persisted, a cell never changes merely because the model or visit order
changes.

Version 1 intentionally omits named-arm asymmetry, the Orion spur, the
Galactic bar and bulge, disk warp and flare, clusters, and correlated multiple
formation. Those can become explicit later distribution versions or named
feature layers. None is needed for the expected reachable region, and none
should be smuggled in through an unversioned formula change.

BBS polities are a separate, explicitly conditioned use of this distribution.
Their required cluster boundary, gateways, TL12 capital neighborhood, and
resolved three-parsec guard volume are specified in
[`bbs-polity-generation.md`](bbs-polity-generation.md). Conditioning a BBS
neighborhood does not modify the background density function. This topology
is eligible only where its complete guard volume lies between 6,000 and
11,000 parsecs in Galactocentric cylindrical radius and passes a separate
local-density and conditioning-likelihood budget.

## Measurement anchors

- [GRAVITY Collaboration: geometric Galactic-center distance](https://arxiv.org/abs/1904.05721)
- [CNS5: local stars and brown dwarfs](https://arxiv.org/abs/2211.01449)
- [Bovy and Rix: stellar-disk radial scale](https://arxiv.org/abs/1309.0809)
- [Bovy et al.: continuous disk scale heights](https://arxiv.org/abs/1111.6585)
- [Reid et al.: maser-based spiral structure](https://arxiv.org/abs/1910.03357)
