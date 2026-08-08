//! Authoritative new-player option catalog and validation.
//!
//! Starter offer IDs and ship IDs are the stable identifiers in
//! `catalog/starting-offers.toml`. Ship prose and crew establishments are
//! read from the admitted catalog records embedded below; no product identity
//! or source PDF is consulted at runtime.

use std::collections::HashSet;

use crate::wire::{
    CaptainCreationOptions, Career, CharacteristicPointBuy, Characteristics, PersonDraft,
    PlayerCreation, ShipSubsystemKind, SkillId, SkillPool, SkillRating, SkillTraining,
    StartingCrewPlan, StartingCrewSlot, StartingOfferTerms, StartingRefitGroup,
    StartingRefitOption, StartingShipOfferSummary, StartingShipOptions, StartingTitleKind,
};

pub const SETUP_REVISION: u64 = 1;
pub const CAPTAIN_CHARACTERISTIC_POINT_BUY: CharacteristicPointBuy = CharacteristicPointBuy {
    minimum: 2,
    maximum: 12,
    neutral: 7,
    budget: 12,
};
pub const CAPTAIN_SKILL_POOL: SkillPool = SkillPool {
    level3: 0,
    level2: 3,
    level1: 6,
    level0: 3,
};
pub const CREW_SKILL_POOL: SkillPool = SkillPool {
    level3: 0,
    level2: 2,
    level1: 4,
    level0: 3,
};

pub const CREW_CHARACTERISTIC_ARRAY: [u8; 6] = [10, 9, 8, 8, 7, 6];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipSubsystemTemplate {
    pub kind: ShipSubsystemKind,
    pub label: String,
    pub maximum_hits: u16,
    pub component_kind: String,
    pub component_id: String,
    pub displacement_millitons: u64,
    pub replacement_price_credits: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipRuntimeComponent {
    pub kind: String,
    pub component_id: String,
    pub quantity: u32,
    pub displacement_millitons: u64,
    pub price_credits: u64,
    pub pack_units: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipWeaponMountSpec {
    pub mount_id: String,
    pub weapons: Vec<String>,
    pub quantity: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipAmmunitionSpec {
    pub ammunition_id: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipCombatSpec {
    pub armor_points: u16,
    pub weapons: Vec<ShipWeaponMountSpec>,
    pub ammunition: Vec<ShipAmmunitionSpec>,
    pub screens: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipStatusSpec {
    pub tech_level: u8,
    pub construction_price_credits: u64,
    pub displacement_millitons: u64,
    pub jump_rating: u8,
    pub thrust_g: u8,
    pub fuel_capacity_millitons: u64,
    pub jump_fuel_millitons: u64,
    pub cargo_capacity_millitons: u64,
    pub passenger_berths: u16,
    pub low_berths: u16,
    pub monthly_life_support_credits: u64,
    pub life_support_capacity_persons: u16,
    pub has_fuel_scoop: bool,
    pub fuel_processing_millitons_per_day: u64,
    pub subsystems: Vec<ShipSubsystemTemplate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipMarketCatalogEntry {
    pub catalog_id: u32,
    pub class_name: String,
    pub price_credits: u64,
    pub cargo_capacity_millitons: u64,
    pub displacement_millitons: u64,
    pub jump_rating: u8,
    pub minimum_crew: u16,
    pub tech_level: u8,
}

struct RuntimeShip {
    catalog_id: u32,
    class_name: &'static str,
    tech_level: u8,
    price_credits: u64,
    displacement_millitons: u64,
    jump_rating: u8,
    thrust_g: u8,
    fuel_millitons: u64,
    jump_fuel_millitons: u64,
    cargo_millitons: u64,
    minimum_crew: u16,
    passenger_accommodation_berths: u16,
    provision_capacity_persons: u16,
    low_berths: u16,
    monthly_life_support_credits: u64,
}

struct RuntimeComponent {
    catalog_id: u32,
    kind: &'static str,
    component_id: &'static str,
    quantity: u32,
    displacement_millitons: u64,
    price_credits: u64,
    pack_units: u32,
}

include!(concat!(env!("OUT_DIR"), "/ship_source_catalog.rs"));

#[derive(Clone)]
struct OfferRecord {
    id: u32,
    path: u8,
    career: Career,
    package: String,
    ship: u32,
    rationale: String,
}

fn offer_records() -> Vec<OfferRecord> {
    include_str!("../../catalog/starting-offers.toml")
        .split("[[offer]]")
        .skip(1)
        .map(|section| {
            let career = match text_value(section, "career").as_str() {
                "trader" => Career::Trader,
                "privateer" => Career::Privateer,
                "navy" => Career::Navy,
                _ => panic!("starter offer has an unknown career"),
            };
            OfferRecord {
                id: scalar_u64(section, "offer_id").expect("starter offer ID") as u32,
                path: scalar_u64(section, "home_path_id").expect("starter path") as u8,
                career,
                package: title_case(&text_value(section, "package_kind")),
                ship: text_value(section, "ship_tag")
                    .strip_prefix("ship-")
                    .expect("starter ship tag")
                    .parse()
                    .expect("numeric starter ship tag"),
                rationale: text_value(section, "selection_rationale"),
            }
        })
        .collect()
}

pub fn home_path(trade_combat: u8, chaos_order: u8) -> u8 {
    let emphasis = if trade_combat < 34 {
        0
    } else if trade_combat < 67 {
        1
    } else {
        2
    };
    let order = if chaos_order >= 67 {
        0
    } else if chaos_order >= 34 {
        1
    } else {
        2
    };
    emphasis * 3 + order + 1
}

pub fn captain_options() -> CaptainCreationOptions {
    let skills = vec![
        rating(SkillId::Leadership, 2),
        rating(SkillId::PilotSpacecraft, 2),
        rating(SkillId::JackOfAllTrades, 2),
        rating(SkillId::Astrogation, 1),
        rating(SkillId::EngineerJump, 1),
        rating(SkillId::EngineerManeuver, 1),
        rating(SkillId::Admin, 1),
        rating(SkillId::Persuade, 1),
        rating(SkillId::Broker, 1),
        rating(SkillId::VaccSuit, 0),
        rating(SkillId::Mechanic, 0),
        rating(SkillId::GunnerTurrets, 0),
    ];
    CaptainCreationOptions {
        setup_revision: SETUP_REVISION,
        characteristic_point_buy: CAPTAIN_CHARACTERISTIC_POINT_BUY,
        skill_pool: CAPTAIN_SKILL_POOL,
        default_captain: PersonDraft {
            name: "Captain".into(),
            characteristics: Characteristics {
                strength: 9,
                dexterity: 9,
                endurance: 9,
                intelligence: 9,
                education: 9,
                charisma: 9,
            },
            training: initial_training(&skills, SkillId::Leadership)
                .expect("Leadership is a trainable captain skill"),
            skills,
        },
    }
}

pub fn offer_summaries(path: u8) -> Vec<StartingShipOfferSummary> {
    offer_records()
        .into_iter()
        .filter(|offer| offer.path == path)
        .map(|offer| offer_summary(&offer))
        .collect()
}

pub fn ship_options(path: u8, revision: u64, offer_id: u32) -> Option<StartingShipOptions> {
    if revision != SETUP_REVISION {
        return None;
    }
    let offer = offer_records()
        .into_iter()
        .find(|offer| offer.path == path && offer.id == offer_id)?;
    Some(StartingShipOptions {
        setup_revision: SETUP_REVISION,
        offer: offer_summary(&offer),
        description_paragraphs: description_paragraphs(ship_source(offer.ship)),
        terms: offer_terms(&offer),
        refit_groups: refit_groups(&offer),
    })
}

fn offer_terms(offer: &OfferRecord) -> StartingOfferTerms {
    let summary = offer_summary(offer);
    let source = ship_source(offer.ship);
    let monthly_life_support = scalar_u64(source, "monthly_life_support_credits").unwrap_or(0);
    let upkeep = summary.price_credits.div_ceil(12_000);
    let crew = u64::from(summary.crew_count).saturating_mul(1_000);
    let refined_fuel = scalar_u64(source, "fuel_millitons")
        .unwrap_or(0)
        .div_ceil(1_000)
        * 500;
    let annual_insurance = summary
        .price_credits
        .saturating_mul(102)
        .div_ceil(10_000)
        .saturating_add(15_000);
    let direct_month = monthly_life_support
        .saturating_add(upkeep)
        .saturating_add(crew)
        .saturating_add(refined_fuel)
        .saturating_add(annual_insurance.div_ceil(12));
    let trade_capital = u64::from(summary.cargo_millitons)
        .div_ceil(1_000)
        .saturating_mul(20_000);
    let (title, equity, principal, liquid, restricted, compensation, authority, exit_terms) =
        match offer.career {
            Career::Trader => (
                StartingTitleKind::OwnedWithLien,
                summary.price_credits / 5,
                summary.price_credits.saturating_mul(4) / 5,
                direct_month.saturating_add(trade_capital),
                0,
                0,
                "Registered owner-master; ordinary commercial and defensive authority",
                "The captain may sell or trade the ship after satisfying its secured lien",
            ),
            Career::Privateer => (
                StartingTitleKind::SponsorOwned,
                0,
                0,
                trade_capital,
                direct_month,
                0,
                "Sponsor command under a limited local commission; prize rights require adjudication",
                "Leaving service requires returning the vessel or acquiring title from the sponsor",
            ),
            Career::Navy => (
                StartingTitleKind::InstitutionOwned,
                0,
                0,
                0,
                direct_month,
                crate::careers::NAVAL_BASE_MONTHLY_SALARY,
                "Commissioned command under naval orders, inspection, pursuit, and defensive authority",
                "The vessel remains institutional property and must be surrendered on separation",
            ),
        };
    StartingOfferTerms {
        terms_revision: SETUP_REVISION,
        title,
        equity_credits: equity,
        principal_credits: principal,
        monthly_payment_credits: if offer.career == Career::Trader { summary.price_credits.div_ceil(240) } else { 0 },
        liquid_reserve_credits: liquid,
        restricted_reserve_credits: restricted,
        monthly_compensation_credits: compensation,
        refit_credit_limit: 20_000_000,
        refit_displacement_millitons: 10_000,
        authority: authority.into(),
        exit_terms: exit_terms.into(),
        insurance: "Comprehensive, crew, passenger, cargo, and public liability; destination assistance is optional".into(),
    }
}

pub fn starting_offer_terms(path: u8, offer_id: u32) -> Option<StartingOfferTerms> {
    offer_records()
        .into_iter()
        .find(|offer| offer.path == path && offer.id == offer_id)
        .map(|offer| offer_terms(&offer))
}

fn refit_groups(offer: &OfferRecord) -> Vec<StartingRefitGroup> {
    let profile = [
        "scheduled-route reserve",
        "escort reserve",
        "dispatch reserve",
        "contested-route reserve",
        "prize-cruise reserve",
        "boarding-patrol reserve",
        "frontier reserve",
        "raiding reserve",
        "auxiliary-patrol reserve",
        "regulated-route reserve",
        "escort-cruise reserve",
        "customs-patrol reserve",
        "independent-route reserve",
        "marque-cruise reserve",
        "anti-raider reserve",
        "blockade-route reserve",
        "syndicate-cruise reserve",
        "local-patrol reserve",
        "fleet-supply reserve",
        "formal-cruise reserve",
        "fleet-patrol reserve",
        "convoy-route reserve",
        "heavy-cruise reserve",
        "corvette-patrol reserve",
        "blockade-running reserve",
        "independent-raider reserve",
        "warlord-patrol reserve",
    ][offer.id.saturating_sub(1) as usize];
    let (name, option_id) = match offer.career {
        Career::Trader => ("Commercial endurance", 2),
        Career::Privateer => ("Cruise endurance", 3),
        Career::Navy => ("Mission endurance", 4),
    };
    let mut options = vec![(
        1,
        "Standard fit",
        "Retain the catalogued ready-to-depart fit",
        0,
        0,
    )];
    if ship_status_spec(offer.ship).is_some_and(|spec| spec.cargo_capacity_millitons >= 10_000) {
        options.push((
            option_id,
            profile,
            "Convert ten tons of the catalogued cargo allocation into permanent jump-fuel tankage",
            0,
            0,
        ));
    }
    vec![StartingRefitGroup {
        group_id: 1,
        name: format!("{name}: {profile}"),
        required: true,
        options: options
            .into_iter()
            .map(
                |(
                    option_id,
                    name,
                    description,
                    displacement_delta_millitons,
                    price_delta_credits,
                )| StartingRefitOption {
                    option_id,
                    name: name.into(),
                    description: description.into(),
                    displacement_delta_millitons,
                    price_delta_credits,
                },
            )
            .collect(),
    }]
}

pub fn validate_refit_options(
    path: u8,
    offer_id: u32,
    selections: &[u32],
) -> Result<(u64, u64), String> {
    let offer = offer_records()
        .into_iter()
        .find(|o| o.path == path && o.id == offer_id)
        .ok_or_else(|| "starting offer is not available".to_string())?;
    if selections.len() != 1 {
        return Err("choose exactly one option from the starting refit group".into());
    }
    let allowed = refit_groups(&offer)
        .into_iter()
        .flat_map(|g| g.options)
        .map(|o| o.option_id)
        .collect::<HashSet<_>>();
    if !allowed.contains(&selections[0]) {
        return Err("starting refit option is not available for this offer".into());
    }
    let extra_fuel = if selections[0] == 1 { 0 } else { 10_000 };
    let spec = ship_status_spec(offer.ship)
        .ok_or_else(|| "starting ship specification is missing".to_string())?;
    if extra_fuel > spec.cargo_capacity_millitons {
        return Err("starting refit does not have ten tons of convertible cargo space".into());
    }
    Ok((extra_fuel, extra_fuel))
}

pub fn offer_catalog_revision(path: u8, offer_id: u32) -> Option<u64> {
    let offer = offer_records()
        .into_iter()
        .find(|offer| offer.path == path && offer.id == offer_id)?;
    scalar_u64(ship_source(offer.ship), "revision")
}

pub fn ship_catalog_revision(catalog_id: u32) -> Option<u64> {
    ship_source_opt(catalog_id).map(|source| scalar_u64(source, "revision").unwrap_or(1))
}

pub fn ship_electronics_dm(catalog_id: u32) -> Option<i8> {
    let source = ship_source_opt(catalog_id)?;
    Some(match text_value(source, "electronics").as_str() {
        "standard" => -4,
        "basic-civilian" => -2,
        "basic-military" => 0,
        "advanced" => 1,
        "very-advanced" => 2,
        _ => return None,
    })
}

pub fn ship_status_spec(catalog_id: u32) -> Option<ShipStatusSpec> {
    let source = ship_source_opt(catalog_id)?;
    let runtime = SHIP_RUNTIME
        .binary_search_by_key(&catalog_id, |ship| ship.catalog_id)
        .ok()
        .map(|index| &SHIP_RUNTIME[index])?;
    Some({
        let construction_price_credits = runtime.price_credits;
        let displacement_millitons = runtime.displacement_millitons;
        let jump_rating = runtime.jump_rating;
        let thrust_g = runtime.thrust_g;
        let jump_fuel_millitons = runtime.jump_fuel_millitons;
        let fuel_capacity_millitons = runtime.fuel_millitons;
        let cargo_capacity_millitons = runtime.cargo_millitons;
        let passenger_berths = runtime.passenger_accommodation_berths;
        let low_berths = runtime.low_berths;
        let monthly_life_support_credits = runtime.monthly_life_support_credits;
        let life_support_capacity_persons = runtime.provision_capacity_persons;
        let hull_configuration = text_value(table_section(source, "[hull]"), "configuration");
        let has_fuel_scoop =
            hull_configuration == "streamlined" || equipment_quantity(source, "fuel-scoop") > 0;
        let fuel_processing_millitons_per_day =
            equipment_quantity(source, "fuel-processor").saturating_mul(20_000);
        ShipStatusSpec {
            tech_level: runtime.tech_level,
            construction_price_credits,
            displacement_millitons,
            jump_rating,
            thrust_g,
            fuel_capacity_millitons,
            jump_fuel_millitons,
            cargo_capacity_millitons,
            passenger_berths,
            low_berths,
            monthly_life_support_credits,
            life_support_capacity_persons,
            has_fuel_scoop,
            fuel_processing_millitons_per_day,
            subsystems: ship_subsystem_templates(catalog_id, source),
        }
    })
}

pub fn ship_runtime_components(catalog_id: u32) -> Vec<ShipRuntimeComponent> {
    SHIP_RUNTIME_COMPONENTS
        .iter()
        .filter(|component| component.catalog_id == catalog_id)
        .map(|component| ShipRuntimeComponent {
            kind: component.kind.to_owned(),
            component_id: component.component_id.to_owned(),
            quantity: component.quantity,
            displacement_millitons: component.displacement_millitons,
            price_credits: component.price_credits,
            pack_units: component.pack_units,
        })
        .collect()
}

/// Materializes the combat-relevant fitted equipment from an admitted ship
/// catalog record.  The returned records retain construction identities so
/// the combat rules catalog, rather than prose or display labels, determines
/// their behavior.
pub fn ship_combat_spec(catalog_id: u32) -> Option<ShipCombatSpec> {
    let source = ship_source_opt(catalog_id)?;
    let armor = optional_table_section(source, "[armor]")
        .and_then(|section| scalar_u64(section, "points"))
        .or_else(|| scalar_u64(source, "armor_points"))
        .unwrap_or(0)
        .try_into()
        .unwrap_or(u16::MAX);
    let mut weapons = Vec::new();
    for section in repeated_table_sections(source, "[[mounts]]") {
        let mount_id = optional_text_value(section, "id")?;
        let fitted = string_array_value(section, "weapons");
        let quantity = scalar_u64(section, "quantity").unwrap_or(1);
        for _ in 0..quantity {
            weapons.push(ShipWeaponMountSpec {
                mount_id: mount_id.clone(),
                weapons: fitted.clone(),
                quantity: 1,
            });
        }
    }
    for header in ["[[barbettes]]", "[[bays]]"] {
        for section in repeated_table_sections(source, header) {
            let weapon_id = optional_text_value(section, "id")?;
            let quantity = scalar_u64(section, "quantity").unwrap_or(1);
            for _ in 0..quantity {
                weapons.push(ShipWeaponMountSpec {
                    mount_id: weapon_id.clone(),
                    weapons: vec![weapon_id.clone()],
                    quantity: 1,
                });
            }
        }
    }
    let ammunition = repeated_table_sections(source, "[[ammunition]]")
        .into_iter()
        .filter_map(|section| {
            Some(ShipAmmunitionSpec {
                ammunition_id: optional_text_value(section, "id")?,
                quantity: scalar_u64(section, "quantity")?
                    .try_into()
                    .unwrap_or(u32::MAX),
            })
        })
        .collect();
    let screens = repeated_table_sections(source, "[[screens]]")
        .into_iter()
        .filter_map(|section| optional_text_value(section, "id"))
        .collect();
    Some(ShipCombatSpec {
        armor_points: armor,
        weapons,
        ammunition,
        screens,
    })
}

pub fn ship_market_catalog() -> Vec<ShipMarketCatalogEntry> {
    SHIP_RUNTIME
        .iter()
        .map(|runtime| ShipMarketCatalogEntry {
            catalog_id: runtime.catalog_id,
            class_name: runtime.class_name.to_owned(),
            price_credits: runtime.price_credits,
            cargo_capacity_millitons: runtime.cargo_millitons,
            displacement_millitons: runtime.displacement_millitons,
            jump_rating: runtime.jump_rating,
            minimum_crew: runtime.minimum_crew,
            tech_level: runtime.tech_level,
        })
        .collect()
}

pub fn crew_plan(path: u8, revision: u64, offer_id: u32) -> Option<StartingCrewPlan> {
    if revision != SETUP_REVISION {
        return None;
    }
    let offer = offer_records()
        .into_iter()
        .find(|offer| offer.path == path && offer.id == offer_id)?;
    let mut roles = crew_roles(ship_source(offer.ship));
    if let Some(command) = roles.iter_mut().find(|(role, _)| role == "command") {
        command.1 = command.1.saturating_sub(1);
    } else if let Some(pilot) = roles.iter_mut().find(|(role, _)| role == "pilot") {
        pilot.1 = pilot.1.saturating_sub(1);
    }
    roles.retain(|(_, count)| *count > 0);
    let slots = roles
        .into_iter()
        .enumerate()
        .map(|(index, (role, count))| StartingCrewSlot {
            slot_id: (index + 1) as u16,
            required: true,
            represented_positions: count,
            skill_pool: CREW_SKILL_POOL,
            default_crew: default_crew(&role, index + 1),
            role,
        })
        .collect();
    Some(StartingCrewPlan {
        setup_revision: SETUP_REVISION,
        starting_offer_id: offer_id,
        slots,
    })
}

pub fn validate_creation(path: u8, creation: &PlayerCreation) -> Result<(), String> {
    if creation.setup_revision != SETUP_REVISION {
        return Err(format!(
            "setup revision {} is stale; current revision is {}",
            creation.setup_revision, SETUP_REVISION
        ));
    }
    validate_person(
        &creation.captain,
        CAPTAIN_CHARACTERISTIC_POINT_BUY,
        CAPTAIN_SKILL_POOL,
        "captain",
    )?;
    if creation.ship_name.trim().is_empty() {
        return Err("ship name must not be blank".into());
    }
    validate_refit_options(path, creation.starting_offer_id, &creation.refit_option_ids)?;
    let plan = crew_plan(path, creation.setup_revision, creation.starting_offer_id)
        .ok_or_else(|| "starting offer is not available for this BBS polity".to_owned())?;
    if creation.crew.len() != plan.slots.len() {
        return Err(format!(
            "offer requires {} initial crew records; received {}",
            plan.slots.len(),
            creation.crew.len()
        ));
    }
    let mut submitted = HashSet::new();
    for crew in &creation.crew {
        if !submitted.insert(crew.slot_id) {
            return Err(format!(
                "crew slot {} was submitted more than once",
                crew.slot_id
            ));
        }
        let slot = plan
            .slots
            .iter()
            .find(|slot| slot.slot_id == crew.slot_id)
            .ok_or_else(|| format!("crew slot {} is not in this offer", crew.slot_id))?;
        validate_name(&crew.name, "crew member")?;
        initial_training(&slot.default_crew.skills, crew.training_skill).ok_or_else(|| {
            format!(
                "{} cannot train {} from this starting skill package",
                slot.role,
                crew.training_skill.name()
            )
        })?;
    }
    Ok(())
}

fn validate_person(
    person: &PersonDraft,
    point_buy: CharacteristicPointBuy,
    pool: SkillPool,
    label: &str,
) -> Result<(), String> {
    validate_name(&person.name, label)?;
    let actual_characteristics = person.characteristics.values();
    if actual_characteristics
        .iter()
        .any(|score| *score < point_buy.minimum || *score > point_buy.maximum)
    {
        return Err(format!(
            "{label} characteristics must be between {} and {}",
            point_buy.minimum, point_buy.maximum
        ));
    }
    let cost = actual_characteristics
        .iter()
        .map(|score| i16::from(*score) - i16::from(point_buy.neutral))
        .sum::<i16>();
    if cost != point_buy.budget {
        return Err(format!(
            "{label} characteristics spend {cost} points; exactly {} are required",
            point_buy.budget
        ));
    }
    validate_skills(person, pool, label)?;
    validate_initial_training(person, label)
}

fn validate_skills(person: &PersonDraft, pool: SkillPool, label: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    let mut counts = [0_u8; 4];
    for rating in &person.skills {
        if !seen.insert(rating.skill) {
            return Err(format!(
                "{label} selects {} more than once",
                rating.skill.name()
            ));
        }
        let index = usize::try_from(rating.level)
            .ok()
            .filter(|level| *level <= 3)
            .ok_or_else(|| format!("{label} has an invalid skill level"))?;
        if rating.skill == SkillId::JackOfAllTrades && !(1..=2).contains(&rating.level) {
            return Err(format!(
                "{label} may select Jack of All Trades only at level 1 or 2"
            ));
        }
        counts[index] += 1;
    }
    if counts != [pool.level0, pool.level1, pool.level2, pool.level3] {
        return Err(format!(
            "{label} skills do not consume the offered rating slots"
        ));
    }
    Ok(())
}

fn validate_name(name: &str, label: &str) -> Result<(), String> {
    if name.trim().is_empty() || name.len() > 128 {
        return Err(format!("{label} name must contain 1..=128 bytes"));
    }
    Ok(())
}

pub fn required_training_weeks(skills: &[SkillRating], skill: SkillId) -> Option<u16> {
    if skill == SkillId::JackOfAllTrades {
        return None;
    }
    let current_level = skills
        .iter()
        .find(|rating| rating.skill == skill)
        .map(|rating| rating.level)?;
    if current_level < 0 {
        return None;
    }
    let skill_total = skills
        .iter()
        .map(|rating| u16::try_from(rating.level).unwrap_or(0))
        .sum::<u16>();
    skill_total.checked_add(u16::try_from(current_level).ok()?.checked_add(1)?)
}

pub fn initial_training(skills: &[SkillRating], skill: SkillId) -> Option<SkillTraining> {
    Some(SkillTraining {
        skill,
        needed_weeks: required_training_weeks(skills, skill)?,
        current_weeks: 0,
    })
}

fn validate_initial_training(person: &PersonDraft, label: &str) -> Result<(), String> {
    let expected =
        required_training_weeks(&person.skills, person.training.skill).ok_or_else(|| {
            format!(
                "{label} training target must be an existing skill other than Jack of All Trades"
            )
        })?;
    if person.training.needed_weeks != expected {
        return Err(format!(
            "{label} training requires {expected} weeks, not {}",
            person.training.needed_weeks
        ));
    }
    if person.training.current_weeks != 0 {
        return Err(format!(
            "{label} must begin with zero completed training weeks"
        ));
    }
    Ok(())
}

fn offer_summary(offer: &OfferRecord) -> StartingShipOfferSummary {
    let source = ship_source(offer.ship);
    let status = ship_status_spec(offer.ship).expect("starting ship has a runtime specification");
    let hull = table_section(source, "[hull]");
    let displacement_tons = text_value(hull, "id")
        .strip_prefix("ship-")
        .expect("ship hull tag")
        .parse()
        .expect("numeric ship hull tag");
    let crew_count = crew_roles(source)
        .iter()
        .map(|(_, count)| u32::from(*count))
        .sum::<u32>()
        .try_into()
        .unwrap_or(u16::MAX);
    StartingShipOfferSummary {
        offer_id: offer.id,
        career: offer.career,
        package_name: offer.package.clone(),
        ship_catalog_id: scalar_u64(source, "catalog_id").expect("ship catalog ID") as u32,
        ship_name: text_value(source, "display_name"),
        role: title_case(&text_value(source, "primary_role")),
        rationale: offer.rationale.clone(),
        displacement_tons,
        jump_rating: scalar_u64(source, "jump_distance").expect("ship jump distance") as u8,
        thrust_g: scalar_u64(source, "thrust_g").expect("ship thrust") as u8,
        cargo_millitons: scalar_u64(source, "cargo_millitons").expect("ship cargo") as u32,
        crew_count,
        price_credits: status.construction_price_credits,
    }
}

fn default_crew_name(role: &str) -> &'static str {
    match role {
        "command" => "Exec",
        "pilot" => "Ace",
        "navigator" => "Nav",
        "engineer" => "Chief",
        "sensors-operator" => "Sensors",
        "screen-operator" => "Screens",
        "turret-gunner" => "Turrets",
        "bay-gunner" => "Bays",
        "gunner" => "Guns",
        "medic" => "Doc",
        "marine" => "Sarge",
        "flight-crew" => "Deck",
        "steward" => "Purser",
        _ => "Bosun",
    }
}

fn default_crew(role: &str, _number: usize) -> PersonDraft {
    let skills = crew_skills(role);
    let levels = [2, 2, 1, 1, 1, 1, 0, 0, 0];
    let skill_ratings = skills
        .into_iter()
        .zip(levels)
        .map(|(skill, level)| rating(skill, level))
        .collect::<Vec<_>>();
    PersonDraft {
        name: default_crew_name(role).into(),
        characteristics: crew_characteristics(role),
        training: initial_training(&skill_ratings, skills[0])
            .expect("the primary crew skill is trainable"),
        skills: skill_ratings,
    }
}

fn crew_characteristics(role: &str) -> Characteristics {
    match role {
        "command" | "steward" => Characteristics {
            strength: 6,
            dexterity: 7,
            endurance: 8,
            intelligence: 9,
            education: 8,
            charisma: 10,
        },
        "pilot" | "flight-crew" => Characteristics {
            strength: 6,
            dexterity: 10,
            endurance: 8,
            intelligence: 9,
            education: 8,
            charisma: 7,
        },
        "navigator" | "sensors-operator" | "screen-operator" => Characteristics {
            strength: 6,
            dexterity: 8,
            endurance: 8,
            intelligence: 10,
            education: 9,
            charisma: 7,
        },
        "engineer" => Characteristics {
            strength: 7,
            dexterity: 8,
            endurance: 8,
            intelligence: 9,
            education: 10,
            charisma: 6,
        },
        "turret-gunner" | "bay-gunner" | "gunner" => Characteristics {
            strength: 7,
            dexterity: 10,
            endurance: 8,
            intelligence: 9,
            education: 8,
            charisma: 6,
        },
        "medic" => Characteristics {
            strength: 6,
            dexterity: 8,
            endurance: 8,
            intelligence: 9,
            education: 10,
            charisma: 7,
        },
        "marine" => Characteristics {
            strength: 10,
            dexterity: 8,
            endurance: 9,
            intelligence: 8,
            education: 7,
            charisma: 6,
        },
        _ => Characteristics {
            strength: 8,
            dexterity: 8,
            endurance: 10,
            intelligence: 7,
            education: 9,
            charisma: 6,
        },
    }
}

fn crew_skills(role: &str) -> [SkillId; 9] {
    match role {
        "command" => [
            SkillId::Leadership,
            SkillId::Admin,
            SkillId::TacticsNaval,
            SkillId::Persuade,
            SkillId::Communications,
            SkillId::Computer,
            SkillId::VaccSuit,
            SkillId::Etiquette,
            SkillId::Carouse,
        ],
        "pilot" => [
            SkillId::PilotSpacecraft,
            SkillId::PilotSmallCraft,
            SkillId::Astrogation,
            SkillId::Communications,
            SkillId::Mechanic,
            SkillId::VaccSuit,
            SkillId::Computer,
            SkillId::Recon,
            SkillId::Admin,
        ],
        "navigator" => [
            SkillId::Astrogation,
            SkillId::Computer,
            SkillId::Communications,
            SkillId::Electronics,
            SkillId::PilotSpacecraft,
            SkillId::VaccSuit,
            SkillId::Admin,
            SkillId::Investigate,
            SkillId::Recon,
        ],
        "engineer" => [
            SkillId::EngineerJump,
            SkillId::EngineerManeuver,
            SkillId::EngineerPower,
            SkillId::EngineerLifeSupport,
            SkillId::Mechanic,
            SkillId::VaccSuit,
            SkillId::Electronics,
            SkillId::Computer,
            SkillId::Admin,
        ],
        "sensors-operator" => [
            SkillId::Communications,
            SkillId::Electronics,
            SkillId::Computer,
            SkillId::Investigate,
            SkillId::Recon,
            SkillId::VaccSuit,
            SkillId::Astrogation,
            SkillId::Admin,
            SkillId::Mechanic,
        ],
        "screen-operator" => [
            SkillId::GunnerScreens,
            SkillId::Electronics,
            SkillId::Computer,
            SkillId::EngineerPower,
            SkillId::VaccSuit,
            SkillId::Mechanic,
            SkillId::Communications,
            SkillId::TacticsNaval,
            SkillId::Recon,
        ],
        "turret-gunner" => [
            SkillId::GunnerTurrets,
            SkillId::Computer,
            SkillId::Electronics,
            SkillId::VaccSuit,
            SkillId::Recon,
            SkillId::Mechanic,
            SkillId::Communications,
            SkillId::TacticsNaval,
            SkillId::PilotSmallCraft,
        ],
        "bay-gunner" => [
            SkillId::GunnerCapital,
            SkillId::GunnerTurrets,
            SkillId::Computer,
            SkillId::Electronics,
            SkillId::VaccSuit,
            SkillId::Mechanic,
            SkillId::Communications,
            SkillId::TacticsNaval,
            SkillId::Recon,
        ],
        "gunner" => [
            SkillId::GunnerTurrets,
            SkillId::GunnerCapital,
            SkillId::Computer,
            SkillId::Electronics,
            SkillId::VaccSuit,
            SkillId::Mechanic,
            SkillId::Communications,
            SkillId::TacticsNaval,
            SkillId::Recon,
        ],
        "flight-crew" => [
            SkillId::PilotSmallCraft,
            SkillId::Mechanic,
            SkillId::PilotSpacecraft,
            SkillId::Electronics,
            SkillId::Communications,
            SkillId::VaccSuit,
            SkillId::GunnerTurrets,
            SkillId::Recon,
            SkillId::EngineerManeuver,
        ],
        "medic" => [
            SkillId::Medicine,
            SkillId::EngineerLifeSupport,
            SkillId::Computer,
            SkillId::Admin,
            SkillId::Persuade,
            SkillId::VaccSuit,
            SkillId::Investigate,
            SkillId::Carouse,
            SkillId::Mechanic,
        ],
        "marine" => [
            SkillId::GunCombat,
            SkillId::TacticsMilitary,
            SkillId::Recon,
            SkillId::VaccSuit,
            SkillId::Melee,
            SkillId::Stealth,
            SkillId::Medicine,
            SkillId::Leadership,
            SkillId::Mechanic,
        ],
        "steward" => [
            SkillId::Etiquette,
            SkillId::Admin,
            SkillId::Persuade,
            SkillId::Carouse,
            SkillId::Broker,
            SkillId::Medicine,
            SkillId::Communications,
            SkillId::Computer,
            SkillId::VaccSuit,
        ],
        _ => [
            SkillId::Mechanic,
            SkillId::VaccSuit,
            SkillId::EngineerLifeSupport,
            SkillId::Communications,
            SkillId::TradeCargomaster,
            SkillId::Admin,
            SkillId::Computer,
            SkillId::Recon,
            SkillId::Medicine,
        ],
    }
}

fn rating(skill: SkillId, level: i8) -> SkillRating {
    SkillRating { skill, level }
}

fn scalar_u64(source: &str, key: &str) -> Option<u64> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn text_value(source: &str, key: &str) -> String {
    source
        .lines()
        .find_map(|line| {
            let (candidate, value) = line.split_once('=')?;
            (candidate.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
        })
        .unwrap_or_else(|| panic!("catalog value {key} is missing"))
}

fn table_section<'a>(source: &'a str, header: &str) -> &'a str {
    let start = source
        .find(header)
        .unwrap_or_else(|| panic!("catalog table {header} is missing"));
    let body = &source[start + header.len()..];
    let end = body.find("\n[").unwrap_or(body.len());
    &body[..end]
}

fn optional_table_section<'a>(source: &'a str, header: &str) -> Option<&'a str> {
    let start = source.find(header)?;
    let body = &source[start + header.len()..];
    let end = body.find("\n[").unwrap_or(body.len());
    Some(&body[..end])
}

fn title_case(value: &str) -> String {
    value
        .split('-')
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn crew_roles(source: &str) -> Vec<(String, u16)> {
    let mut result = Vec::new();
    let mut in_crew = false;
    let mut role = None;
    let mut quantity = 1;
    for line in source.lines().chain(["[[end]]"]) {
        let line = line.trim();
        if line.starts_with("[[") {
            if in_crew && let Some(role) = role.take() {
                result.push((role, quantity));
            }
            in_crew = line == "[[crew]]";
            quantity = 1;
            continue;
        }
        if !in_crew {
            continue;
        }
        if let Some(value) = line.strip_prefix("role = ") {
            role = Some(value.trim_matches('"').to_owned());
        } else if let Some(value) = line.strip_prefix("quantity = ") {
            quantity = value.parse().unwrap_or(1);
        }
    }
    result
}

fn description_paragraphs(source: &str) -> Vec<String> {
    let Some(start) = source.find("description_paragraphs =") else {
        return Vec::new();
    };
    let source = &source[start..];
    let Some(open) = source.find('[') else {
        return Vec::new();
    };
    let source = &source[open + 1..];
    let Some(close) = source.find("]\n") else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut value = String::new();
    for character in source[..close].chars() {
        if quoted {
            if escaped {
                value.push(character);
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                values.push(std::mem::take(&mut value));
                quoted = false;
            } else {
                value.push(character);
            }
        } else if character == '"' {
            quoted = true;
        }
    }
    values
}

fn ship_subsystem_templates(catalog_id: u32, source: &str) -> Vec<ShipSubsystemTemplate> {
    let displacement_tons = ship_displacement_millitons(source) / 1000;
    let armor = optional_table_section(source, "[armor]").unwrap_or("");
    let mut result = vec![
        subsystem(
            ShipSubsystemKind::Hull,
            "Hull",
            scalar_u64(source, "hull_points").unwrap_or((displacement_tons / 50).max(1)),
        ),
        subsystem(
            ShipSubsystemKind::Structure,
            "Structure",
            scalar_u64(source, "structure_points").unwrap_or((displacement_tons / 50).max(1)),
        ),
        subsystem(
            ShipSubsystemKind::Armor,
            "Armor",
            scalar_u64(source, "armor_points")
                .or_else(|| scalar_u64(armor, "points"))
                .unwrap_or(0),
        ),
        subsystem(ShipSubsystemKind::Bridge, "Bridge", 3),
        subsystem(ShipSubsystemKind::Computer, "Computer", 3),
        subsystem(ShipSubsystemKind::Sensors, "Sensors", 3),
    ];
    if scalar_u64(source, "jump_rating")
        .or_else(|| scalar_u64(table_section(source, "[fuel]"), "jump_distance"))
        .unwrap_or(0)
        > 0
    {
        result.push(subsystem(ShipSubsystemKind::JumpDrive, "Jump drive", 3));
    }
    result.extend([
        subsystem(ShipSubsystemKind::ManeuverDrive, "Maneuver drive", 3),
        subsystem(ShipSubsystemKind::PowerPlant, "Power plant", 3),
        subsystem(ShipSubsystemKind::FuelSystem, "Fuel system", 3),
        subsystem(ShipSubsystemKind::LifeSupport, "Life support", 3),
        subsystem(ShipSubsystemKind::CargoHold, "Cargo hold", 3),
    ]);
    append_catalog_subsystems(
        &mut result,
        source,
        "[[equipment]]",
        ShipSubsystemKind::Other,
        false,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[parameterized_equipment]]",
        ShipSubsystemKind::Other,
        false,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[mounts]]",
        ShipSubsystemKind::WeaponMount,
        true,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[barbettes]]",
        ShipSubsystemKind::WeaponMount,
        true,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[bays]]",
        ShipSubsystemKind::WeaponMount,
        true,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[screens]]",
        ShipSubsystemKind::Screen,
        true,
    );
    append_catalog_subsystems(
        &mut result,
        source,
        "[[hangars]]",
        ShipSubsystemKind::Hangar,
        true,
    );
    hydrate_subsystem_components(catalog_id, &mut result);
    result
}

fn ship_displacement_millitons(source: &str) -> u64 {
    scalar_u64(source, "hull_millitons").unwrap_or_else(|| {
        text_value(table_section(source, "[hull]"), "id")
            .strip_prefix("ship-")
            .expect("ship hull tag")
            .parse::<u64>()
            .expect("numeric ship hull tag")
            * 1000
    })
}

fn subsystem(
    kind: ShipSubsystemKind,
    label: impl Into<String>,
    maximum_hits: u64,
) -> ShipSubsystemTemplate {
    ShipSubsystemTemplate {
        kind,
        label: label.into(),
        maximum_hits: u16::try_from(maximum_hits).unwrap_or(u16::MAX),
        component_kind: String::new(),
        component_id: String::new(),
        displacement_millitons: 0,
        replacement_price_credits: 0,
    }
}

fn hydrate_subsystem_components(catalog_id: u32, subsystems: &mut [ShipSubsystemTemplate]) {
    let components = ship_runtime_components(catalog_id);
    let mut occurrence = std::collections::HashMap::<(String, String), u32>::new();
    for subsystem in subsystems {
        let candidates: &[&str] = match subsystem.kind {
            ShipSubsystemKind::Hull | ShipSubsystemKind::Structure => &["hull"],
            ShipSubsystemKind::Armor => &["armor"],
            ShipSubsystemKind::Bridge => &["bridge"],
            ShipSubsystemKind::Computer => &["computer"],
            ShipSubsystemKind::Sensors => &["electronics"],
            ShipSubsystemKind::JumpDrive => &["jump-drive"],
            ShipSubsystemKind::ManeuverDrive => &["maneuver-drive"],
            ShipSubsystemKind::PowerPlant => &["power-plant"],
            ShipSubsystemKind::FuelSystem => &["fuel"],
            ShipSubsystemKind::LifeSupport => &["equipment"],
            ShipSubsystemKind::CargoHold => &[],
            ShipSubsystemKind::WeaponMount => &["weapon-mount", "barbette", "bay"],
            ShipSubsystemKind::Screen => &["screen"],
            ShipSubsystemKind::Hangar => &["hangar"],
            ShipSubsystemKind::Other => &["equipment", "parameterized-equipment"],
        };
        let normalized_label = subsystem
            .label
            .split(" group")
            .next()
            .unwrap_or(&subsystem.label)
            .trim_end_matches(|character: char| character.is_ascii_digit() || character == ' ')
            .to_ascii_lowercase()
            .replace(' ', "-");
        let wanted_id = if subsystem.component_id.is_empty() {
            normalized_label.as_str()
        } else {
            subsystem.component_id.as_str()
        };
        let found = components.iter().find(|component| {
            candidates.contains(&component.kind.as_str())
                && (component.component_id == wanted_id
                    || candidates.len() == 1
                        && !matches!(subsystem.kind, ShipSubsystemKind::Other)
                        && !matches!(subsystem.kind, ShipSubsystemKind::WeaponMount))
        });
        if let Some(component) = found {
            let key = (component.kind.clone(), component.component_id.clone());
            let seen = occurrence.entry(key).or_default();
            *seen = seen.saturating_add(1);
            let divisor = if matches!(subsystem.kind, ShipSubsystemKind::WeaponMount) {
                u64::from(component.quantity.max(1))
            } else {
                1
            };
            subsystem.component_kind = component.kind.clone();
            subsystem.component_id = component.component_id.clone();
            subsystem.displacement_millitons = component.displacement_millitons / divisor;
            subsystem.replacement_price_credits = component.price_credits / divisor;
        } else if subsystem.kind == ShipSubsystemKind::CargoHold {
            subsystem.component_kind = "cargo".into();
            subsystem.component_id = "cargo-hold".into();
        } else {
            subsystem.component_kind = format!("{:?}", subsystem.kind).to_ascii_lowercase();
            subsystem.component_id = normalized_label;
        }
    }
}

fn append_catalog_subsystems(
    target: &mut Vec<ShipSubsystemTemplate>,
    source: &str,
    header: &str,
    kind: ShipSubsystemKind,
    expand_quantity: bool,
) {
    for section in repeated_table_sections(source, header) {
        let Some(id) = optional_text_value(section, "id") else {
            continue;
        };
        let quantity = scalar_u64(section, "quantity").unwrap_or(1);
        let count = if expand_quantity { quantity } else { 1 };
        for instance in 0..count {
            let mut label = title_case(&id);
            if expand_quantity && quantity > 1 {
                label.push_str(&format!(" {}", instance + 1));
            } else if !expand_quantity && quantity > 1 {
                label.push_str(&format!(" group ({quantity})"));
            }
            let mut template = subsystem(kind, label, 3);
            template.component_id = id.clone();
            template.component_kind = match header {
                "[[equipment]]" => "equipment",
                "[[parameterized_equipment]]" => "parameterized-equipment",
                "[[mounts]]" => "weapon-mount",
                "[[barbettes]]" => "barbette",
                "[[bays]]" => "bay",
                "[[screens]]" => "screen",
                "[[hangars]]" => "hangar",
                _ => "other",
            }
            .into();
            target.push(template);
        }
    }
}

fn repeated_table_sections<'a>(source: &'a str, header: &str) -> Vec<&'a str> {
    source
        .split(header)
        .skip(1)
        .map(|tail| {
            let end = tail.find("\n[").unwrap_or(tail.len());
            &tail[..end]
        })
        .collect()
}

fn equipment_quantity(source: &str, wanted_id: &str) -> u64 {
    repeated_table_sections(source, "[[equipment]]")
        .into_iter()
        .filter(|section| optional_text_value(section, "id").as_deref() == Some(wanted_id))
        .map(|section| scalar_u64(section, "quantity").unwrap_or(1))
        .sum()
}

fn optional_text_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
    })
}

fn string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(line) = source.lines().find(|line| {
        line.split_once('=')
            .is_some_and(|(candidate, _)| candidate.trim() == key)
    }) else {
        return Vec::new();
    };
    let Some((_, value)) = line.split_once('=') else {
        return Vec::new();
    };
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

fn ship_source_opt(id: u32) -> Option<&'static str> {
    SHIP_SOURCES
        .binary_search_by_key(&id, |(catalog_id, _)| *catalog_id)
        .ok()
        .map(|index| SHIP_SOURCES[index].1)
}

fn ship_source(id: u32) -> &'static str {
    ship_source_opt(id).expect("starting offer references unknown ship")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_crew(person: &PersonDraft, label: &str) {
        let mut actual = person.characteristics.values();
        let mut expected = CREW_CHARACTERISTIC_ARRAY;
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected, "{label} characteristic array");
        validate_skills(person, CREW_SKILL_POOL, label).unwrap();
    }

    #[test]
    fn every_matrix_cell_has_three_offers_and_valid_defaults() {
        let mut refit_names = std::collections::HashSet::new();
        let mut refit_group_names = std::collections::HashSet::new();
        for path in 1..=9 {
            let offers = offer_summaries(path);
            assert_eq!(offers.len(), 3);
            for offer in offers {
                assert_eq!(offer.jump_rating, 2);
                let status = ship_status_spec(offer.ship_catalog_id).unwrap();
                assert!(status.displacement_millitons >= 100_000);
                assert_eq!(status.jump_rating, offer.jump_rating);
                assert_eq!(status.thrust_g, offer.thrust_g);
                assert!(status.fuel_capacity_millitons >= status.jump_fuel_millitons);
                assert!(
                    status
                        .subsystems
                        .iter()
                        .any(|system| system.kind == ShipSubsystemKind::Hull)
                );
                assert!(
                    status
                        .subsystems
                        .iter()
                        .any(|system| system.kind == ShipSubsystemKind::JumpDrive)
                );
                let plan = crew_plan(path, SETUP_REVISION, offer.offer_id).unwrap();
                assert!(!plan.slots.is_empty());
                for slot in plan.slots {
                    assert_valid_crew(&slot.default_crew, "crew");
                }

                let details = ship_options(path, SETUP_REVISION, offer.offer_id).unwrap();
                assert_eq!(details.terms.terms_revision, SETUP_REVISION);
                assert_eq!(details.refit_groups.len(), 1);
                assert!(details.refit_groups[0].required);
                assert!(refit_group_names.insert(details.refit_groups[0].name.clone()));
                assert_eq!(
                    validate_refit_options(path, offer.offer_id, &[1]).unwrap(),
                    (0, 0)
                );
                if status.cargo_capacity_millitons >= 10_000 {
                    assert_eq!(details.refit_groups[0].options.len(), 2);
                    assert!(refit_names.insert(details.refit_groups[0].options[1].name.clone()));
                    let alternate = details.refit_groups[0].options[1].option_id;
                    assert_eq!(
                        validate_refit_options(path, offer.offer_id, &[alternate]).unwrap(),
                        (10_000, 10_000)
                    );
                } else {
                    assert_eq!(details.refit_groups[0].options.len(), 1);
                }
                assert!(validate_refit_options(path, offer.offer_id, &[]).is_err());

                match offer.career {
                    Career::Trader => {
                        assert_eq!(details.terms.title, StartingTitleKind::OwnedWithLien);
                        assert!(
                            details.terms.equity_credits > 0,
                            "trader offer {} has no financed hull value",
                            offer.offer_id
                        );
                        assert!(
                            details.terms.principal_credits > 0,
                            "trader offer {} has no financed principal",
                            offer.offer_id
                        );
                        assert_eq!(details.terms.restricted_reserve_credits, 0);
                    }
                    Career::Privateer => {
                        assert_eq!(details.terms.title, StartingTitleKind::SponsorOwned);
                        assert_eq!(details.terms.principal_credits, 0);
                        assert!(details.terms.restricted_reserve_credits > 0);
                    }
                    Career::Navy => {
                        assert_eq!(details.terms.title, StartingTitleKind::InstitutionOwned);
                        assert_eq!(details.terms.principal_credits, 0);
                        assert!(details.terms.restricted_reserve_credits > 0);
                        assert!(details.terms.monthly_compensation_credits > 0);
                    }
                }
            }
        }
        assert_eq!(refit_group_names.len(), 27);
        assert!(!refit_names.is_empty());
        validate_person(
            &captain_options().default_captain,
            CAPTAIN_CHARACTERISTIC_POINT_BUY,
            CAPTAIN_SKILL_POOL,
            "captain",
        )
        .unwrap();
    }

    #[test]
    fn ship_market_catalog_includes_non_starter_progression_hulls() {
        let catalog = ship_market_catalog();
        assert!(catalog.len() > 100);
        assert!(
            catalog
                .iter()
                .any(|entry| entry.displacement_millitons >= 5_000_000)
        );
        assert!(catalog.iter().any(|entry| {
            !offer_records()
                .iter()
                .any(|offer| offer.ship == entry.catalog_id)
        }));
    }

    #[test]
    fn every_catalog_crew_role_has_a_standard_template() {
        for role in [
            "bay-gunner",
            "command",
            "engineer",
            "flight-crew",
            "gunner",
            "marine",
            "medic",
            "navigator",
            "other",
            "pilot",
            "screen-operator",
            "sensors-operator",
            "steward",
            "turret-gunner",
        ] {
            let crew = default_crew(role, 1);
            assert_valid_crew(&crew, role);
            assert_eq!(crew.skills.len(), 9);
            assert_eq!(crew.name, default_crew_name(role));
            assert!(!crew.name.starts_with("Crew "));
            assert_eq!(crew.training.skill, crew.skills[0].skill);
            assert_eq!(crew.training.needed_weeks, 11);
            assert_eq!(crew.training.current_weeks, 0);
        }
    }

    #[test]
    fn initial_training_uses_the_ce_skill_total_formula() {
        let captain = captain_options().default_captain;
        assert_eq!(captain.training.skill, SkillId::Leadership);
        assert_eq!(captain.training.needed_weeks, 15);
        assert_eq!(captain.training.current_weeks, 0);
        assert_eq!(
            required_training_weeks(&captain.skills, SkillId::Astrogation),
            Some(14)
        );
        assert_eq!(
            required_training_weeks(&captain.skills, SkillId::VaccSuit),
            Some(13)
        );
        assert_eq!(
            required_training_weeks(&captain.skills, SkillId::JackOfAllTrades),
            None
        );

        let mut invalid = captain.clone();
        invalid.training.needed_weeks -= 1;
        assert!(
            validate_person(
                &invalid,
                CAPTAIN_CHARACTERISTIC_POINT_BUY,
                CAPTAIN_SKILL_POOL,
                "captain"
            )
            .unwrap_err()
            .contains("requires 15 weeks")
        );

        let mut progressed = captain;
        progressed.training.current_weeks = 1;
        assert!(
            validate_person(
                &progressed,
                CAPTAIN_CHARACTERISTIC_POINT_BUY,
                CAPTAIN_SKILL_POOL,
                "captain"
            )
            .unwrap_err()
            .contains("zero completed")
        );
    }

    #[test]
    fn catalog_descriptions_and_crew_are_embedded() {
        let ships = offer_records()
            .into_iter()
            .map(|offer| offer.ship)
            .collect::<HashSet<_>>();
        assert_eq!(ships.len(), 19);
        for ship in ships {
            assert!(!description_paragraphs(ship_source(ship)).is_empty());
            assert!(!crew_roles(ship_source(ship)).is_empty());
        }
    }

    #[test]
    fn captain_characteristics_use_the_authoritative_point_buy() {
        let mut captain = captain_options().default_captain;
        captain.characteristics = Characteristics {
            strength: 12,
            dexterity: 12,
            endurance: 9,
            intelligence: 9,
            education: 6,
            charisma: 6,
        };
        validate_person(
            &captain,
            CAPTAIN_CHARACTERISTIC_POINT_BUY,
            CAPTAIN_SKILL_POOL,
            "captain",
        )
        .unwrap();

        captain.characteristics.charisma = 5;
        assert!(
            validate_person(
                &captain,
                CAPTAIN_CHARACTERISTIC_POINT_BUY,
                CAPTAIN_SKILL_POOL,
                "captain",
            )
            .unwrap_err()
            .contains("exactly 12")
        );

        captain.characteristics.charisma = 13;
        assert!(
            validate_person(
                &captain,
                CAPTAIN_CHARACTERISTIC_POINT_BUY,
                CAPTAIN_SKILL_POOL,
                "captain",
            )
            .unwrap_err()
            .contains("between 2 and 12")
        );
    }

    #[test]
    fn jack_of_all_trades_is_limited_to_levels_one_and_two() {
        let default_captain = captain_options().default_captain;
        validate_person(
            &default_captain,
            CAPTAIN_CHARACTERISTIC_POINT_BUY,
            CAPTAIN_SKILL_POOL,
            "captain",
        )
        .unwrap();

        let mut level_one_captain = default_captain.clone();
        level_one_captain
            .skills
            .iter_mut()
            .find(|rating| rating.skill == SkillId::JackOfAllTrades)
            .unwrap()
            .skill = SkillId::Computer;
        level_one_captain
            .skills
            .iter_mut()
            .find(|rating| rating.level == 1)
            .unwrap()
            .skill = SkillId::JackOfAllTrades;
        validate_person(
            &level_one_captain,
            CAPTAIN_CHARACTERISTIC_POINT_BUY,
            CAPTAIN_SKILL_POOL,
            "captain",
        )
        .unwrap();

        for level in [0, 3] {
            let mut captain = default_captain.clone();
            captain
                .skills
                .iter_mut()
                .find(|rating| rating.skill == SkillId::JackOfAllTrades)
                .unwrap()
                .level = level;
            assert!(
                validate_person(
                    &captain,
                    CAPTAIN_CHARACTERISTIC_POINT_BUY,
                    CAPTAIN_SKILL_POOL,
                    "captain",
                )
                .unwrap_err()
                .contains("only at level 1 or 2")
            );
        }
    }

    #[test]
    fn streamlined_starting_hulls_receive_their_implicit_ce_fuel_scoops() {
        for offer in offer_records() {
            let source = ship_source(offer.ship);
            if text_value(table_section(source, "[hull]"), "configuration") == "streamlined" {
                assert!(ship_status_spec(offer.ship).unwrap().has_fuel_scoop);
            }
        }
    }

    #[test]
    fn runtime_accommodation_uses_people_for_provisions_and_rooms_for_passage() {
        let trafalgar = ship_status_spec(180).unwrap();
        assert_eq!(trafalgar.life_support_capacity_persons, 322);
        assert_eq!(trafalgar.passenger_berths, 12);
        assert_eq!(trafalgar.monthly_life_support_credits, 171_000);

        let crusoe = ship_status_spec(193).unwrap();
        assert_eq!(crusoe.life_support_capacity_persons, 24);
        assert_eq!(crusoe.passenger_berths, 8);
        assert_eq!(crusoe.low_berths, 12);
    }
}
