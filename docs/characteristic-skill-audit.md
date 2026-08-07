# Characteristic and Skill Audit

**Status:** rules analysis, not an adopted character model. Core CE remains
the default where this document identifies a differing Clement rule. Each
change of vocabulary or mechanics requires a separate design decision.

## Purpose

This audit identifies the character characteristics and skills that have a
mechanical use in Cepheus Trader. It is not a starting-character package and
does not import the Cepheus career/lifepath game. Its immediate purposes are
to:

- choose one canonical vocabulary for captain and crew records;
- identify the competencies used by trade, ship operation, combat, boarding,
  passenger, port, crime, and naval rules;
- separate directly useful skills from subsystem-specific and RPG-only
  skills; and
- determine what a captain may need when substituting for an absent crew
  member.

The audit compares the core CE rules in `cepodnew-markdown/` with the local
third-edition Clement material, especially *Clement Sector Third Edition*,
*Bounded Fortune*, *Port of Entry*, *Skull and Crossbones Third Edition*,
*Hub Federation Navy Third Edition*, and *Anderson & Felix Third Edition*.
Where vocabularies or mechanics differ, the audit records the difference and
a possible normalization; it does not adopt the Clement variant merely
because it is newer. Setting-specific Z-drive references still describe the
corresponding standard CE Jump-drive task for purposes of determining whether
a competency is applicable.

## Dispositions

The tables use these classifications:

- **Direct** — already has a mechanical use in the planned normal trade,
  ship-operation, combat, boarding, passenger, port, crime, or naval loops.
  It belongs in the initial character/crew rules model, although not every
  character needs it.
- **Conditional** — valid and useful when a named optional or later subsystem
  is active. Preserve the vocabulary, but do not build unrelated gameplay
  merely to justify it.
- **Omit** — has no current simulator use beyond RPG characterization,
  lifepath events, or an unrelated minigame. Do not put it into a starter
  package or initial rules implementation.
- **Alias** — an older CE name translated to a newer skill. It must not become
  a second independently trainable skill.

Classification is about the game rules, not the breadth of a particular
captain. A direct skill may belong principally to a hired specialist.

## Characteristics

Cepheus Trader uses the five ordinary characteristics common to both rules
vocabularies and adopts Clement's Charisma as the sixth:

| Characteristic | Disposition | Applicable uses |
| --- | --- | --- |
| Strength (STR) | Direct | Personal and boarding combat, lifting and emergency work, physical cargo handling, and one of the three CE damage tracks |
| Dexterity (DEX) | Direct | Spacecraft and small-craft piloting, gunnery, personal attacks, forced docking, hull breaching, initiative, and cargo handling |
| Endurance (END) | Direct | First CE damage track, fatigue, vacuum/environmental survival, recovery, low-berth survival, sustained maintenance, security, and cargo work |
| Intelligence (INT) | Direct | Tactical command, electronic warfare and hacking, investigation, mechanics, medical judgment, and commercial decisions such as auctions |
| Education (EDU) | Direct | Astrogation, Jump initiation, engineering, sensors, repair, medicine, communications, and many formal commercial/administrative tasks |
| Social Standing (SOC) | Replaced | Core CE uses it for trade and social checks, institutional position, and titles; Cepheus Trader does not store it as a characteristic |
| Charisma (CHA) | Direct; adopted | Leadership, bargaining, crew relations, passengers, officials, diplomacy, surrender, corruption, and criminal contacts |
| Psionic Strength (PSI) | Omit | No adopted subsystem uses psionics |

### Social Standing Versus Charisma

*Clement Sector Third Edition* explicitly replaces Social Standing with
Charisma and directs older references to convert SOC to CHA one-for-one.
Cepheus Trader adopts that change because:

- CHA measures the personal influence used in a task.
- Naval rank, office, title, citizenship, guild membership, warrants,
  reputation, and relationships are separate persistent facts.
- Those facts are scoped to a polity, institution, market, or person rather
  than becoming one universal social score.

An old CE `Broker + SOC` or `Liaison + SOC` check therefore becomes an
appropriate social skill plus CHA. A captain can be a senior officer at home,
an unknown foreigner elsewhere, and a wanted criminal in a third polity
without changing characteristics.

All six adopted non-psionic characteristics remain mechanically relevant.
Dropping STR, DEX, or END would require abandoning CE personal injury and
boarding rather than merely simplifying character creation.

## Skill and Specialization Semantics

The sources contain two incompatible specialization models.

Core CE uses standard cascade skills. Whenever a cascade is received,
including at level 0, the character immediately selects the specific skill:

- `Pilot (Spacecraft) 0` does not grant `Pilot (Small Craft) 0`;
- `Engineer (Jump Drive) 0` does not grant other Engineer competencies; and
- `Gunner (Turrets) 0` does not grant Screens or Capital Weapons.

*Clement Sector Third Edition* instead has a family-level base skill at level
0. The first positive level selects a specialization, and positive levels in
different specializations advance separately. This gives broader familiarity
before specialization and reduces the number of emergency tasks made at
DM-3.

Core cascade semantics remain the current baseline. The Clement base-plus-
specialization model is a candidate simplification, especially for small-crew
cross-training, but requires an explicit decision before implementation. The
two must not be accidentally combined.

Under either model, no skill means the ordinary untrained DM-3. Jack of All
Trades reduces that penalty by one per level, grants no positive skill bonus,
and is capped at 3. Multiple simultaneous duties still receive CE's DM-2 per
additional action, and mutually simultaneous combat stations cannot be
covered merely by listing several skills on one person.

## Direct Shipboard and Commercial Roles

The following is the operational cross-reference supplied principally by
core CE and *Bounded Fortune*. Z-drive has been normalized to Jump drive.

| Role or task | Major skill | Common characteristic or supporting skills |
| --- | --- | --- |
| Captain | Leadership | CHA; Admin, Persuade, Tactics, and the skills of any doubled billet |
| Executive officer | Admin and Leadership | CHA; department skills |
| Pilot, 100 tons or more | Pilot (Spacecraft) | DEX; Astrogation, Sensors, Tactics (Naval) |
| Small-craft pilot | Pilot (Small Craft) | DEX; Astrogation, Sensors, Tactics (Naval) |
| Astrogator and Jump plot | Astrogation | EDU; Computers and Sensors |
| Jump initiation | Engineer (Jump Drive) | EDU |
| Sensor operator | Electronics (Sensors) | EDU or INT; Computers |
| Communications operator | Electronics (Communications) | EDU or CHA; Computers, Diplomat, Etiquette |
| Electronic intrusion | Electronics (Computers) | INT |
| Ship engineer | Engineer (Maneuver Drive, Jump Drive, Life Support, or Power) | INT or EDU; Electrical Repair, Mechanic |
| Damage control and general repair | Mechanic | EDU for core combat damage control; END or INT for routine work |
| Drone or missile operator | Electronics (Remote Operations) | DEX, INT, or EDU according to the operation |
| Turret or small-craft gunner | Gunner (Turrets) | DEX; Sensors, Remote Operations, Tactics (Naval) |
| Bay or spinal gunner | Gunner (Capital Weapons) | DEX; Sensors, Tactics (Naval) |
| Screen operator | Gunner (Screens) | EDU or INT |
| Cargomaster | Trade (Cargomaster) | STR, DEX, or END; Admin, Computers, Sensors, Mechanic |
| Broker | Broker | CHA for negotiation, INT for pricing/auction judgment, EDU for formal searches |
| Steward/purser | Etiquette or Admin | CHA; Chef, Persuade, Broker |
| Chef | Chef | DEX or END |
| Ship's doctor | Medic (First Aid or Diagnosis) | INT or EDU; Cryogenics for low berths |
| Security/boarder | Gun Combat or Melee | DEX for shooting, STR or DEX for melee; Tactics (Military), Suit, Freefall |

The captain may double any of these roles for which the captain has the
competence, but that fact does not create extra actions. A lone captain can
pilot during normal flight and later negotiate a cargo purchase; the same
captain cannot pilot, operate sensors, perform electronic warfare, and fire
three turrets simultaneously in an encounter.

For late-game capital ships, *Anderson & Felix Third Edition* supplies a
useful scaling rule: store aggregate crew strength and crew skill for the
rank-and-file, while individually modeling the captain and important
department heads. Its crew-strength penalties, firing-frequency limits, and
average skill DM avoid creating thousands of character records. A level-4
commanding officer or department head can provide the source's limited
officer bonus. This is compatible with, rather than a replacement for, the
individual model used on starter ships.

## Complete Clement-Vocabulary Skill Audit

This table uses the newer labels to make its specialization boundaries clear.
It is an applicability inventory, not adoption of the Clement vocabulary. The
translation table following it shows how the same competencies relate to the
core CE names.

| Skill | Disposition | Applicable use or reason |
| --- | --- | --- |
| Admin | Direct | Ship administration, contracts, port bureaucracy, payroll, executive-officer work, navy logistics, and one accepted passenger-service major skill |
| Advocate | Direct in Legal; Conditional in Oratory/Politics | Warrants, customs, prize title, arrests, regulations, and formal appeals use Legal. Oratory and Politics apply only if mass persuasion or polity-political missions become player actions. |
| Animals | Conditional | Live-animal cargo, veterinary handling, farming, riding, or animal-related planetary contracts; not ordinary freight |
| Athletics | Conditional | Chases, climbing, thrown attacks, and physical boarding or surface encounters |
| Art | Omit | No current operational rule; retain only if art cargo/expertise or entertainment becomes a real subsystem |
| Astrogation | Direct | In-system course plotting and standard CE Jump plotting; replaces space uses of old Navigation |
| Broker | Direct | Suppliers, buyers, speculative price, freight/passenger acquisition, auctions, leases, loans, insurance, and ship transactions |
| Carouse | Conditional | Port information, recruiting, contacts, and crew/passenger morale if those are exposed as actions |
| Chef | Direct | Passenger-service and crew-service quality; replaces the food-service part of old Steward |
| Deception | Direct in Forgery and Lie; Conditional in Disguise, Intrusion, Pickpocket | False manifests, identity/title fraud, smuggling, accepting illicit payments, and piracy use Forgery/Lie. Disguise and physical intrusion need encounter support; Pickpocket is RPG-scale. |
| Diplomat | Direct | Official negotiation, cross-polity naval contact, access disputes, peaceful resolution, and surrender terms |
| Discipline | Direct | The Clement limited-use modifier to trained actions under combat pressure |
| Draw | Conditional | Personal-combat initiative only; not used by ship combat |
| Drive | Conditional | Ground-vehicle portions of planetary contracts and boarding follow-up |
| Electronics (Communications) | Direct | Port control, mail/data exchange, signals, jamming, counter-jamming, and communications protocol |
| Electronics (Computers) | Direct | Ship computers, networks, security systems, door hacking, software, data retrieval, and automated systems |
| Electronics (Electrical Repair) | Direct | Electrical installation, repair, refit, and damage control |
| Electronics (Remote Operations) | Direct | Drones, remote craft, missiles, repair devices, and telepresence |
| Electronics (Robotics) | Conditional | Robot modification/repair when robots are individually modeled; remote operation alone uses Remote Operations |
| Electronics (Sensors) | Direct | Detection, identification, targeting support, navigation support, inspections, and detecting deception such as ship holomasks |
| Engineer (Maneuver Drive) | Direct | M-drive operation, maintenance, failures, and refueling-related engineering |
| Engineer (Jump Drive) | Direct | Standard J-drive operation, maintenance, Jump initiation, and failures; canonical replacement for Zimm Drive |
| Engineer (Life Support) | Direct | Ship habitability, passenger accommodation, damage, and maintenance |
| Engineer (Power) | Direct | Power-plant operation, maintenance, allocation, and damage |
| Etiquette | Direct | Passenger service, formal officials, naval protocol, and the social/service part of old Steward |
| Explosives | Conditional | Sabotage, demolition, breaching, mines, and bomb disposal during missions or boarding |
| Flyer | Conditional | Atmospheric craft used in planetary missions; spacecraft and orbital small craft use Pilot |
| Gambler | Omit | No current operational rule; a captain's-club minigame would be separate scope |
| Gun Combat | Direct | Shipboard security, boarding, piracy, mutiny, and personal defense; use the weapon specialization actually carried |
| Gunner (Capital Weapons) | Direct | Bays and spinal mounts on later naval/private ships |
| Gunner (Ortillery) | Conditional | Planetary bombardment or fire support only |
| Gunner (Screens) | Direct | Defensive screen operation on equipped ships |
| Gunner (Turrets) | Direct | Turrets, barbettes where specified, and small-craft weapons |
| Heavy Weapons | Conditional | Marine, boarding, or ground actions with the relevant carried weapon |
| Instruction | Conditional | Accelerated crew training if onboard instruction becomes part of progression |
| Interrogation (Questioning) | Conditional | Prisoners, pirate intelligence, customs, and naval intelligence; Torture is not required by any planned loop |
| Investigate | Direct | Customs inspection, evidence, bounty/warrant work, fraud, cargo tampering, and anomaly analysis |
| Jack of All Trades | Direct | Reduces the untrained penalty and is particularly relevant when a small-ship captain fills an emergency billet |
| Language | Conditional | Regional languages, first contact, and cross-polity communication if translation is not assumed |
| Leadership | Direct | Captaincy, orders, team assistance, initiative, morale, and department command |
| Mechanic | Direct | General maintenance, combat damage control, hull work, forced boarding, and non-drive equipment |
| Medic (Cryogenics) | Direct if low berths exist | Safe low-berth operation and revival |
| Medic (Diagnosis) | Direct | Ship's-doctor role and treatment decisions |
| Medic (First Aid) | Direct | Injury stabilization and immediate care |
| Medic (Surgery) | Conditional | Serious injury when the ship or facility has a sickbay; normally a specialist rather than an emergency captain duty |
| Medic (Alteration, Altrants, Cybernetics, Uplifts) | Conditional | Add only with the corresponding species/body-modification subsystem; Clement setting assumptions do not automatically enter this universe |
| Melee | Direct | Boarding, shipboard security, piracy, mutiny, and unarmed emergencies; use the relevant specialization |
| Navigation | Conditional | Surface and sea navigation only in the newer vocabulary; never use it for Jump or in-system space courses |
| Persuade | Direct | Hiring, passenger and contract negotiation, surrender, compliance, corruption, and the newer rules' bribery check |
| Pilot (Small Craft) | Direct | Pinnaces, cutters, fighters, ship's boats, and other sub-100-ton craft |
| Pilot (Spacecraft) | Direct | Ships of 100 tons or more, including docking, landing, maneuver, skimming, and forced docking |
| Recon | Direct | Threat spotting, boarding reconnaissance, inspection, ambush avoidance, and mission scouting |
| Science | Conditional | Use a named specialty only when a rule or installed system calls for it, such as Physics for advanced gunnery/engineering, Economics for analysis, or Psychology for passengers/crew |
| Seafarer | Conditional | Surface boats and submarines during planetary operations; a starship landing in water still uses Pilot (Spacecraft) |
| Stealth | Direct | Boarding, covert inspection, smuggling, piracy, and infiltration |
| Streetwise | Direct | Illicit suppliers/buyers, fences, criminal intelligence, corrupt contacts, and local underworld risk |
| Suit (Vacc Suit) | Direct | EVA, decompression, exterior repair, skimming emergencies, and boarding |
| Suit (Battle Armor) | Conditional | Armored marine or boarding operations |
| Suit (Hostile Environment) | Conditional | Hazardous planetary destinations and surface contracts |
| Survival (Freefall) | Direct | Work and combat in microgravity/freefall; canonical replacement for old Zero-G |
| Survival (other environments) | Conditional | Planetary survival and rescue contracts in the named environment |
| Tactics (Military) | Direct | Boarding, security teams, marines, and abstract boarding command |
| Tactics (Naval) | Direct | Ship combat, fleet/squadron coordination, mission planning, and command initiative |
| Tactics (Sport) | Omit | No operational use |
| Trade (Cargomaster) | Direct | Cargo loading, securing, manifests, custody, loss prevention, mail, and efficient cargo operations |
| Trade (Naval Architect, Prospector, Space Construction, and similar) | Conditional | Ship design, resource exploration, facilities, or production only when those become player actions |
| Trade (Bartender and ordinary crafts) | Omit | No current operational use except on a deliberately modeled passenger-service or production vessel |

## Candidate Core-to-Clement Normalization

These mappings would avoid duplicate competencies if the newer vocabulary is
adopted. Until then, the core name and cascade semantics remain authoritative.

| Core CE name | Candidate Clement-style translation |
| --- | --- |
| Social Standing | Charisma for personal checks; rank/title/reputation/authority become scoped persistent state |
| Navigation, when used in space | Astrogation |
| Piloting | Pilot (Spacecraft) or Pilot (Small Craft) |
| Comms | Electronics (Communications) for transmission/EW; Electronics (Sensors) for detection/targeting |
| Computer | Electronics (Computers) |
| Electronics | The applicable Electronics specialization |
| Engineering | The applicable Engineer specialization |
| Gravitics | Engineer (Maneuver Drive), Electronics, or Mechanic according to the device/task; no blanket alias |
| Gunnery; Bay Weapons; Spinal Mounts; Screens; Turret Weapons | The corresponding Gunner specialization; bays/spinals become Capital Weapons |
| Medicine | The applicable Medic specialization |
| Mechanics | Mechanic |
| Steward | Etiquette for service/social duties, Chef for food, or Admin where *Bounded Fortune* permits it as the steward major skill |
| Zero-G | Survival (Freefall); vacc-suit operation is separately Suit (Vacc Suit) |
| Battle Dress | Suit (Battle Armor) |
| Bribery | Persuade + CHA to make the offer; Streetwise may find the contact and Deception may conceal participation |
| Liaison | Diplomat for official relations or Persuade for individual influence |
| Linguistics | Language |
| Demolitions | Explosives |
| Prospecting | Trade (Prospector) |
| Life, Physical, Social, or Space Sciences | A named Science specialization |
| Old personal-weapon skills | The corresponding Gun Combat or Melee specialization |
| Old vehicle and watercraft cascade skills | Drive, Flyer, or Seafarer and the applicable specialization |
| Veterinary Medicine | Animals (Veterinary), unless treating a supported uplift species invokes a specific Medic specialty |

## Source Checks Behind the Classification

- Core CE Chapter 6 makes the Jump plot an EDU-based Navigation task and Jump
  initiation an EDU-based Engineering task. Under the newer names these are
  Astrogation and Engineer (Jump Drive).
- Core CE Chapters 5 and 10 directly use DEX, all three physical damage
  characteristics, Piloting, Comms, Mechanics, Tactics, and personal weapon
  skills. Those map to the direct ship-combat and boarding entries above.
- Core CE Chapter 7 uses Broker with EDU, INT, or SOC; Computer for online
  suppliers; and Streetwise for illegal suppliers. Clement's current trade
  procedure instead uses Broker + CHA, Streetwise + CHA, Persuade + CHA for a
  customs bribe, and Deception (Forgery) + EDU for false cargo documents.
- *Bounded Fortune*, pages 56–68, explicitly lists small-crew role doubling,
  each major crew skill, useful supporting skills, and characteristics. It
  specifically says a captain may also be owner, pilot, astrogator,
  cargomaster, sensor operator, broker, or steward.
- *Skull and Crossbones Third Edition*, pages 28–30, makes forced docking a
  Pilot + DEX task, weak-hull location Mechanic + EDU, hull cutting Mechanic +
  DEX, and hatch hacking Electronics (Computers) + INT. Detecting its
  starship holomask uses Electronics (Sensors) + INT.
- *Anderson & Felix Third Edition*, page 111, defines aggregate capital-ship
  crew strength and skill plus a limited bonus from level-4 commanding
  officers or department heads.
- *Clement Sector Third Edition*'s appendix explicitly changes SOC to CHA,
  removes Steward in favor of Chef and Etiquette, folds Zero-G into Survival
  (Freefall), and moves Vacc Suit and Battle Suit under Suit.

## Consequences for Captain Design

1. The captain record needs STR, DEX, END, INT, EDU, CHA, and a sparse skill
   set. SOC is not stored as a characteristic; socially scoped facts are
   separate state. Standard cascade versus base-plus-specialization remains
   unresolved. PSI is unnecessary unless psionics are later adopted.
2. A captain needs Leadership to satisfy the sourced captain role, but “can
   stand in for any crew role” does not mean “starts trained in every role.”
   Under core CE, specifically selected level-0 cascade skills and Jack of All
   Trades provide emergency breadth. The Clement alternative would make
   family-level 0 the broader familiarity mechanism.
3. Each starting offer should let the captain choose or receive one meaningful
   doubled operational billet. The hired crew must still make the ship legally
   and practically operable after that choice.
4. A captain working two concurrent encounter stations takes the normal
   multiple-action penalty when physically possible; truly simultaneous
   stations require another crew member, automation, or software.
5. Personal combat, boarding, injury, and EVA make the physical
   characteristics and their associated skills real game state. If a future
   design removes those systems, it should revisit the classifications rather
   than leave unused character statistics behind.
6. Capital-ship scale should transition from individually stored ordinary
   crew to aggregate department strength/skill, retaining individual officers
   only where their decisions and source-defined bonuses matter.

This audit deliberately stops before selecting the competing specialization
models or assigning starting values, point costs, caps, or career-specific
packages. Those require explicit design decisions based on this mechanically
applicable set. Published-character measurements relevant to those decisions
are recorded in `docs/character-creation-benchmarks.md`.
