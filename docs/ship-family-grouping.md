# Ship Design Family Grouping

*Status: current, 2026-07-27*

The active 213-design catalog is grouped into 113 stable-numbered design
families.
The authoritative machine-readable relationship is
[`catalog/ships/families.toml`](../catalog/ships/families.toml), and every
`ship-N.toml` record repeats its numeric `family_id`. Validation requires the
two representations to agree.

This pass assigns relationships only. Canonical names are layered through
`catalog/ships/names.toml`; manufacturers, shipyards, upgrade paths, and
progression positions remain separate relationships.

## Grouping Rules

A family requires evidence of a shared design lineage:

- an explicit common platform or hull;
- a published variant, conversion, production block, or model generation;
- the same design repeated in more than one source; or
- a standard carried-craft chassis repeated with different mission fits.

Equal displacement, similar statistics, or a common role is not enough. Two
unrelated 400-ton patrol ships remain separate even if normalization makes
their bills of materials similar. Conversely, a commercial fit and a naval
fit remain in one family when both are documented derivatives of the same
platform.

A stable family ID is anchored to the lowest-numbered current member.
Thus `family-54` means only “the family whose earliest catalog member is
`ship-54`.” The number has no setting, manufacturer, capability, progression,
or rank meaning and remains stable if a player-facing name later changes.

A singleton is a complete one-design family, not an unclassified record. It
may later be merged only when actual lineage evidence is found, not merely to
make an upgrade path appear complete.

## Shared Lineages

The 39 multi-design families contain 139 catalog designs:

| Family | Members | PI-free grouping rationale |
| --- | --- | --- |
| `family-1` | `ship-1`, `ship-2`, `ship-3`, `ship-4` | Common 10-ton work-pod chassis with courier, mining, transfer, and maintenance fits |
| `family-5` | `ship-5`, `ship-158` | Same compact transfer-launch design in separate source records |
| `family-6` | `ship-6`, `ship-176`, `ship-177` | Common aerodynamic fast-launch platform with standard, command, and flag-transport fits |
| `family-7` | `ship-7`, `ship-8`, `ship-145`, `ship-146`, `ship-185`, `ship-189` | Standard flattened-cylinder fast-launch line in utility and armed fits |
| `family-9` | `ship-9`, `ship-11` | Two production blocks of one compact carrier-fighter platform |
| `family-12` | `ship-12`, `ship-15` | Related production fits of one aerospace-fighter platform |
| `family-17` | `ship-17`, `ship-19`, `ship-178`, `ship-179` | Common aerodynamic 30-ton ship's-boat platform with personnel and high-capacity fits |
| `family-18` | `ship-18`, `ship-134`, `ship-165`, `ship-181`, `ship-187`, `ship-212` | Standard 30-ton utility/boarding boat repeated across parent-ship sources |
| `family-20` | `ship-20`, `ship-21`, `ship-24`, `ship-159`, `ship-188`, `ship-209` | Standard 50-ton modular cutter chassis with cargo, passenger, mining, medical, survey-base, and generic module fits |
| `family-22` | `ship-22`, `ship-151` | Same utility-pinnace design in separate source records |
| `family-26` | `ship-26`, `ship-33` | Common 100-ton dispatch hull in mail and covert-service fits |
| `family-27` | `ship-27`, `ship-28`, `ship-31` | Same 100-ton light-trader design repeated across sources |
| `family-30` | `ship-30`, `ship-32` | Same heavy assault-lander design repeated across sources |
| `family-34` | `ship-34`, `ship-43` | Same 200-ton independent prospecting design repeated across sources |
| `family-35` | `ship-35`, `ship-36` | Same 80-ton missile attack-boat design repeated across sources |
| `family-38` | `ship-38`, `ship-39`, `ship-40` | Common 200-ton trader hull with freight, standard-passenger, and steerage fits |
| `family-45` | `ship-45`, `ship-46` | Same 200-ton free-trader design represented by two source records |
| `family-48` | `ship-48`, `ship-55`, `ship-56` | Common 95-ton attack-boat architecture in torpedo, fast-attack, and sensor/reconnaissance fits |
| `family-49` | `ship-49`, `ship-50`, `ship-51`, `ship-52` | Common 300-ton cargo hull with two production generations and interstellar/in-system fits |
| `family-54` | `ship-54`, `ship-57`, `ship-58`, `ship-59`, `ship-60`, `ship-62`, `ship-65`, `ship-161`, `ship-162`, `ship-163`, `ship-164`, `ship-166`, `ship-167`, `ship-168` | Broad 300-ton merchant lineage spanning commercial, passenger, bulk-cargo, entertainment, security, missile, and later armed-refit designs |
| `family-68` | `ship-68`, `ship-69`, `ship-72`, `ship-139`, `ship-140`, `ship-141` | Common 400-ton platform offered in yacht, marauder, and armed-escort fits, including repeated source records |
| `family-78` | `ship-78`, `ship-79` | Common 800-ton expedition hull in prospecting and troop-transport fits |
| `family-80` | `ship-80`, `ship-82`, `ship-83`, `ship-86`, `ship-88`, `ship-152`, `ship-153`, `ship-154`, `ship-155`, `ship-156`, `ship-157` | Common 800-ton modular freighter line spanning standard, extended-range, passenger, colony, armed-merchant, and missile fits |
| `family-90` | `ship-90`, `ship-96` | A 550-ton system-defense conversion explicitly derived from the interstellar sloop platform |
| `family-91` | `ship-91`, `ship-95` | Same 600-ton patrol-corvette design repeated across sources |
| `family-93` | `ship-93`, `ship-160` | Same 1,000-ton hospital-ship design repeated across editions/sources |
| `family-97` | `ship-97`, `ship-98`, `ship-101` | Common 2,000-ton heavy-freighter platform in tanker, freight, and transport fits |
| `family-99` | `ship-99`, `ship-107` | Same 1,500-ton destroyer lineage represented by two source records |
| `family-100` | `ship-100`, `ship-102` | Baseline and improved fits of one 1,000-ton frigate platform |
| `family-104` | `ship-104`, `ship-105` | Common 1,400-ton modular hull used as an attack-boat tender and strike carrier |
| `family-116` | `ship-116`, `ship-118` | Baseline and alternate-service fits of one 2,500-ton cruiser platform |
| `family-117` | `ship-117`, `ship-119`, `ship-120` | Successive production blocks of one carrier lineage |
| `family-121` | `ship-121`, `ship-122` | Interstellar escort and system-defense boat explicitly constructed on one 500-ton common hull |
| `family-126` | `ship-126`, `ship-127` | Common 1,900-ton freight platform in interstellar and in-system fits |
| `family-128` | `ship-128`, `ship-129`, `ship-130` | Common 600-ton fast-trader platform converted for replenishment and forward deployment |
| `family-131` | `ship-131`, `ship-132`, `ship-133` | Original and later drive/refit generations of one 400-ton merchant design |
| `family-135` | `ship-135`, `ship-136`, `ship-137`, `ship-138` | Common 1,000-ton destroyer chassis with general, battle, direct-fire, and missile mission fits |
| `family-147` | `ship-147`, `ship-148`, `ship-149`, `ship-150`, `ship-190` | Common 300-ton modular merchant platform with cargo, mixed-passenger, passenger, and long-range survey fits |
| `family-169` | `ship-169`, `ship-170`, `ship-171`, `ship-172`, `ship-173`, `ship-174`, `ship-175` | Two generations of one 100-ton light-trader lineage with austere, maximum-cargo, passenger, dispatch, and extended-range fits |

## Singleton Families

The remaining 74 families currently contain one design each:

`family-10`, `family-13`, `family-14`, `family-16`, `family-23`,
`family-25`, `family-29`, `family-37`, `family-41`, `family-42`,
`family-44`, `family-47`, `family-53`, `family-61`, `family-63`,
`family-64`, `family-66`, `family-67`, `family-70`, `family-71`,
`family-73`, `family-74`, `family-75`, `family-76`, `family-77`,
`family-81`, `family-84`, `family-85`, `family-87`, `family-89`,
`family-92`, `family-94`, `family-103`, `family-106`, `family-108`,
`family-109`, `family-110`, `family-111`, `family-112`, `family-113`,
`family-114`, `family-115`, `family-123`, `family-124`, `family-125`,
`family-142`, `family-143`, `family-144`, `family-180`, `family-182`,
`family-183`, `family-184`, `family-186`, `family-191`, `family-192`,
`family-193`, `family-194`, `family-195`, `family-196`, `family-197`,
`family-198`, `family-199`, `family-200`, `family-201`, `family-202`,
`family-203`, `family-204`, `family-205`, `family-206`, `family-207`,
`family-208`, `family-210`, `family-211`, and `family-213`.

## Related Catalog Data

Native path assignments are in `catalog/ships/upgrade-paths.toml`; canonical
family and design names are in `catalog/ships/names.toml`. Several variants
from one family may occupy different paths, and a sparse path may explicitly
backfill a gap with a design from an adjacent path. Neither path assignments
nor naming may alter family membership merely to make a progression ladder
look regular.
