//! Cepheus Trader's authoritative merchant rules.
//!
//! Generic commodity data and availability come from the revised Open Game
//! Content table in *Bounded Fortune*.  Proper names from its optional colour
//! tables are intentionally absent.  Negotiation uses the Clement task
//! outcomes instead of the core CE extreme-percentage table.

use crate::crypto::{CryptoError, SeedStream, derive_seed};
use crate::universe::{Starport, World};

pub const REFINED_FUEL_PRICE_PER_TON: u64 = 500;
pub const MILLITONS_PER_TON: u64 = 1_000;
pub const STARTING_RESERVE_REFERENCE_PRICE_PER_TON: u64 = 20_000;
pub const FREIGHT_RATE_PER_TON_PARSEC: u64 = 3_500;
pub const HIGH_PASSAGE_RATE_PER_PARSEC: u64 = 25_000;
pub const MIDDLE_PASSAGE_RATE_PER_PARSEC: u64 = 10_000;
pub const STEERAGE_PASSAGE_RATE_PER_PARSEC: u64 = 5_000;
pub const LOW_PASSAGE_RATE: u64 = 2_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TradeCode {
    Agricultural,
    Asteroid,
    FluidOceans,
    Garden,
    HighPopulation,
    HighTechnology,
    IceCapped,
    Industrial,
    NonAgricultural,
    NonIndustrial,
    Poor,
    Rich,
    Vacuum,
}

impl TradeCode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Agricultural => "Ag",
            Self::Asteroid => "As",
            Self::FluidOceans => "Fl",
            Self::Garden => "Ga",
            Self::HighPopulation => "Hi",
            Self::HighTechnology => "Ht",
            Self::IceCapped => "Ic",
            Self::Industrial => "In",
            Self::NonAgricultural => "Na",
            Self::NonIndustrial => "Ni",
            Self::Poor => "Po",
            Self::Rich => "Ri",
            Self::Vacuum => "Va",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TradeModifier {
    pub code: TradeCode,
    pub dm: i8,
}

const NONE: TradeModifier = TradeModifier {
    code: TradeCode::Vacuum,
    dm: 0,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuantityDice {
    pub dice: u8,
    pub sides: u8,
    pub multiplier_tons: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommodityDefinition {
    pub id: u16,
    pub d66: u8,
    pub name: &'static str,
    pub base_price_per_ton: u64,
    pub quantity: QuantityDice,
    pub purchase_modifiers: [TradeModifier; 2],
    pub sale_modifiers: [TradeModifier; 2],
    pub common: bool,
}

const fn q(dice: u8, multiplier_tons: u16) -> QuantityDice {
    QuantityDice {
        dice,
        sides: 6,
        multiplier_tons,
    }
}

const fn m(code: TradeCode, dm: i8) -> TradeModifier {
    TradeModifier { code, dm }
}

const fn good(
    id: u16,
    d66: u8,
    name: &'static str,
    price: u64,
    quantity: QuantityDice,
    purchase: [TradeModifier; 2],
    sale: [TradeModifier; 2],
) -> CommodityDefinition {
    CommodityDefinition {
        id,
        d66,
        name,
        base_price_per_ton: price,
        quantity,
        purchase_modifiers: purchase,
        sale_modifiers: sale,
        common: false,
    }
}

pub const COMMON_GOODS: [CommodityDefinition; 6] = [
    CommodityDefinition {
        id: 1,
        d66: 0,
        name: "Basic Consumable Goods",
        base_price_per_ton: 1_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
    CommodityDefinition {
        id: 2,
        d66: 0,
        name: "Basic Electronics",
        base_price_per_ton: 25_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
    CommodityDefinition {
        id: 3,
        d66: 0,
        name: "Basic Machine Parts",
        base_price_per_ton: 10_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
    CommodityDefinition {
        id: 4,
        d66: 0,
        name: "Basic Manufactured Goods",
        base_price_per_ton: 20_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
    CommodityDefinition {
        id: 5,
        d66: 0,
        name: "Basic Raw Materials",
        base_price_per_ton: 5_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
    CommodityDefinition {
        id: 6,
        d66: 0,
        name: "Basic Unrefined Ore",
        base_price_per_ton: 2_000,
        quantity: q(2, 5),
        purchase_modifiers: [NONE; 2],
        sale_modifiers: [NONE; 2],
        common: true,
    },
];

// Revised Bounded Fortune table.  Result 66 is deliberately not a generic
// commodity: unusual objects need an individually materialized record.
pub const TRADE_GOODS: [CommodityDefinition; 35] = [
    good(
        11,
        11,
        "Electronics",
        100_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 1), m(TradeCode::Industrial, 2)],
        [m(TradeCode::NonIndustrial, 1), m(TradeCode::Poor, 1)],
    ),
    good(
        12,
        12,
        "Sporting Equipment",
        5_500,
        q(2, 5),
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::HighPopulation, 2),
            m(TradeCode::NonIndustrial, 2),
        ],
    ),
    good(
        13,
        13,
        "Agricultural Equipment",
        150_000,
        q(1, 1),
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 1)],
        [m(TradeCode::Agricultural, 2), m(TradeCode::Garden, 1)],
    ),
    good(
        14,
        14,
        "Animal Products",
        1_500,
        q(4, 5),
        [m(TradeCode::Agricultural, 1), m(TradeCode::Garden, 2)],
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        15,
        15,
        "Collectibles",
        50_000,
        q(1, 1),
        [m(TradeCode::Industrial, 1), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::HighPopulation, 1),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        16,
        16,
        "Computers and Handcomps",
        150_000,
        q(2, 1),
        [m(TradeCode::HighTechnology, 2), m(TradeCode::Industrial, 1)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        17,
        21,
        "Crystals and Gems",
        20_000,
        q(1, 5),
        [
            m(TradeCode::NonIndustrial, 2),
            m(TradeCode::NonAgricultural, 1),
        ],
        [m(TradeCode::Industrial, 1), m(TradeCode::Rich, 1)],
    ),
    good(
        18,
        22,
        "Cybernetics",
        250_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 2), m(TradeCode::Rich, 1)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        19,
        23,
        "Food Service Equipment",
        4_000,
        q(2, 1),
        [
            m(TradeCode::Industrial, 2),
            m(TradeCode::NonAgricultural, 1),
        ],
        [
            m(TradeCode::Agricultural, 1),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        20,
        24,
        "Furniture",
        5_000,
        q(4, 1),
        [m(TradeCode::Agricultural, 1), m(TradeCode::Garden, 2)],
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        21,
        25,
        "Gambling Equipment",
        4_000,
        q(1, 1),
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Rich, 1)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        22,
        26,
        "Vehicles",
        160_000,
        q(1, 1),
        [m(TradeCode::HighTechnology, 2), m(TradeCode::Rich, 1)],
        [m(TradeCode::NonIndustrial, 2), m(TradeCode::Poor, 1)],
    ),
    good(
        23,
        31,
        "Grocery Products",
        6_000,
        q(1, 5),
        [m(TradeCode::Agricultural, 3), m(TradeCode::Garden, 2)],
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        24,
        32,
        "Household Appliances",
        12_000,
        q(4, 1),
        [m(TradeCode::HighPopulation, 2), m(TradeCode::Industrial, 3)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 2),
        ],
    ),
    good(
        25,
        33,
        "Industrial Supplies",
        75_000,
        q(2, 1),
        [m(TradeCode::Industrial, 3), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 2),
        ],
    ),
    good(
        26,
        34,
        "Liquor and Other Intoxicants",
        15_000,
        q(1, 5),
        [m(TradeCode::Agricultural, 2), m(TradeCode::Garden, 1)],
        [m(TradeCode::Industrial, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        27,
        35,
        "Luxury Goods and Rarities",
        150_000,
        q(1, 1),
        [m(TradeCode::Agricultural, 1), m(TradeCode::Garden, 2)],
        [m(TradeCode::Industrial, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        28,
        36,
        "Manufacturing Equipment",
        750_000,
        q(1, 5),
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::NonAgricultural, 1),
            m(TradeCode::NonIndustrial, 2),
        ],
    ),
    good(
        29,
        41,
        "Medical Equipment",
        50_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 2), m(TradeCode::Rich, 2)],
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Industrial, 2)],
    ),
    good(
        30,
        42,
        "Petrochemicals",
        10_000,
        q(2, 5),
        [
            m(TradeCode::NonAgricultural, 2),
            m(TradeCode::NonIndustrial, 2),
        ],
        [m(TradeCode::Agricultural, 1), m(TradeCode::Industrial, 2)],
    ),
    good(
        31,
        43,
        "Pharmaceuticals",
        100_000,
        q(1, 1),
        [m(TradeCode::HighTechnology, 3), NONE],
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 1)],
    ),
    good(
        32,
        44,
        "Polymers",
        7_000,
        q(4, 5),
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 1)],
        [m(TradeCode::NonIndustrial, 2), m(TradeCode::Vacuum, 1)],
    ),
    good(
        33,
        45,
        "Precious Metals",
        50_000,
        q(1, 1),
        [m(TradeCode::Asteroid, 3), m(TradeCode::IceCapped, 2)],
        [m(TradeCode::Industrial, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        34,
        46,
        "Radioactive Ore",
        1_000_000,
        q(1, 1),
        [m(TradeCode::Asteroid, 2), m(TradeCode::NonIndustrial, 3)],
        [m(TradeCode::Industrial, 2), m(TradeCode::HighTechnology, 1)],
    ),
    good(
        35,
        51,
        "Robots and Drones",
        500_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 3), m(TradeCode::Industrial, 2)],
        [m(TradeCode::NonIndustrial, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        36,
        52,
        "Scientific Equipment",
        50_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 3), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::HighPopulation, 2),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        37,
        53,
        "Survival Gear",
        4_000,
        q(2, 1),
        [m(TradeCode::Garden, 2), m(TradeCode::Rich, 2)],
        [m(TradeCode::FluidOceans, 2), m(TradeCode::Vacuum, 1)],
    ),
    good(
        38,
        54,
        "Textiles",
        3_000,
        q(3, 5),
        [
            m(TradeCode::Agricultural, 3),
            m(TradeCode::NonIndustrial, 2),
        ],
        [m(TradeCode::NonAgricultural, 1), m(TradeCode::Rich, 2)],
    ),
    good(
        39,
        55,
        "Construction Supplies",
        20_000,
        q(2, 5),
        [
            m(TradeCode::Agricultural, 3),
            m(TradeCode::NonIndustrial, 2),
        ],
        [
            m(TradeCode::Industrial, 2),
            m(TradeCode::NonAgricultural, 1),
        ],
    ),
    good(
        40,
        56,
        "Raw Materials",
        20_000,
        q(2, 5),
        [m(TradeCode::Asteroid, 2), m(TradeCode::Vacuum, 1)],
        [
            m(TradeCode::Industrial, 2),
            m(TradeCode::NonAgricultural, 1),
        ],
    ),
    good(
        41,
        61,
        "Live Animals",
        25_000,
        q(5, 5),
        [m(TradeCode::Agricultural, 3), m(TradeCode::Garden, 2)],
        [m(TradeCode::HighPopulation, 1), m(TradeCode::Industrial, 2)],
    ),
    good(
        42,
        62,
        "Children's Toys",
        5_000,
        q(2, 5),
        [m(TradeCode::Industrial, 2), m(TradeCode::Rich, 2)],
        [
            m(TradeCode::HighPopulation, 2),
            m(TradeCode::NonIndustrial, 1),
        ],
    ),
    good(
        43,
        63,
        "Medical Laboratory Equipment",
        50_000,
        q(1, 5),
        [m(TradeCode::HighTechnology, 2), m(TradeCode::Rich, 3)],
        [
            m(TradeCode::Industrial, 2),
            m(TradeCode::NonAgricultural, 2),
        ],
    ),
    good(
        44,
        64,
        "Military Supplies",
        150_000,
        q(2, 1),
        [m(TradeCode::HighTechnology, 3), m(TradeCode::Industrial, 2)],
        [
            m(TradeCode::HighPopulation, 2),
            m(TradeCode::NonIndustrial, 2),
        ],
    ),
    good(
        45,
        65,
        "Personal Weapons and Armor",
        30_000,
        q(2, 1),
        [m(TradeCode::Industrial, 3), m(TradeCode::Rich, 2)],
        [m(TradeCode::NonIndustrial, 2), m(TradeCode::Poor, 2)],
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Legality {
    Legal,
    Restricted,
    Prohibited,
}

pub fn commodity(id: u16) -> Option<CommodityDefinition> {
    COMMON_GOODS
        .iter()
        .chain(TRADE_GOODS.iter())
        .copied()
        .find(|item| item.id == id)
}

/// Returns one of the actual catalogued commodities by a dense zero-based
/// index. Commodity identifiers deliberately mirror the source table and are
/// therefore not a dense numeric range.
pub fn commodity_by_index(index: u64) -> Option<CommodityDefinition> {
    let common_len = COMMON_GOODS.len() as u64;
    if index < common_len {
        return COMMON_GOODS.get(index as usize).copied();
    }
    TRADE_GOODS.get((index - common_len) as usize).copied()
}

pub const COMMODITY_COUNT: u64 = (COMMON_GOODS.len() + TRADE_GOODS.len()) as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplierLot {
    pub commodity: CommodityDefinition,
    pub quantity_millitons: u64,
}

/// Materializes the individual lots revealed by one supplier search.  The
/// result follows the Bounded Fortune starport lot tables; an optional player
/// maximum causes the broker to keep canvassing within the same search until
/// each returned lot fits.
pub fn supplier_lots(
    system_seed: [u8; 32],
    assignment_id: u64,
    effect: i16,
    maximum_quantity_millitons: u64,
    world: &World,
) -> Result<Vec<SupplierLot>, CryptoError> {
    if effect <= -6 || world.starport == Starport::X {
        return Ok(Vec::new());
    }
    let mut label = Vec::from(&b"commerce/supplier-search/v1"[..]);
    label.extend_from_slice(&assignment_id.to_be_bytes());
    let mut random = SeedStream::new(derive_seed(system_seed, &label)?);
    let (common_count_dice, common_size_dice, trade_count_dice) = match world.starport {
        Starport::A => (6, 2, 4),
        Starport::B => (4, 1, 3),
        Starport::C => (2, 1, 2),
        Starport::D | Starport::E => (1, 1, 1),
        Starport::X => unreachable!(),
    };
    let multiplier = if effect >= 6 { 2 } else { 1 };
    let common_count = roll_dice(&mut random, common_count_dice, 6)? * multiplier;
    let trade_count = if effect < 0 {
        0
    } else {
        roll_dice(&mut random, trade_count_dice, 6)? * multiplier
    };
    let mut lots = Vec::with_capacity((common_count + trade_count) as usize);
    for _ in 0..common_count {
        let item = COMMON_GOODS[(random.next_u64()? % COMMON_GOODS.len() as u64) as usize];
        if let Some(quantity_millitons) = fitting_quantity(
            &mut random,
            QuantityDice {
                dice: common_size_dice,
                sides: 6,
                multiplier_tons: 10,
            },
            maximum_quantity_millitons,
        )? {
            lots.push(SupplierLot {
                commodity: item,
                quantity_millitons,
            });
        }
    }
    for _ in 0..trade_count {
        let item = TRADE_GOODS[(random.next_u64()? % TRADE_GOODS.len() as u64) as usize];
        if let Some(quantity_millitons) =
            fitting_quantity(&mut random, item.quantity, maximum_quantity_millitons)?
        {
            lots.push(SupplierLot {
                commodity: item,
                quantity_millitons,
            });
        }
    }
    Ok(lots)
}

pub fn targeted_supplier_lot(
    system_seed: [u8; 32],
    assignment_id: u64,
    item: CommodityDefinition,
    maximum_quantity_millitons: u64,
) -> Result<Option<SupplierLot>, CryptoError> {
    let mut label = Vec::from(&b"commerce/private-supplier-search/v1"[..]);
    label.extend_from_slice(&assignment_id.to_be_bytes());
    let mut random = SeedStream::new(derive_seed(system_seed, &label)?);
    Ok(
        fitting_quantity(&mut random, item.quantity, maximum_quantity_millitons)?.map(
            |quantity_millitons| SupplierLot {
                commodity: item,
                quantity_millitons,
            },
        ),
    )
}

pub fn buyer_lot_quantity(
    system_seed: [u8; 32],
    assignment_id: u64,
    item: CommodityDefinition,
    maximum_quantity_millitons: u64,
    world: &World,
) -> Result<Option<u64>, CryptoError> {
    let dice = if item.common {
        QuantityDice {
            dice: match world.starport {
                Starport::A => 2,
                Starport::B | Starport::C | Starport::D | Starport::E => 1,
                Starport::X => return Ok(None),
            },
            sides: 6,
            multiplier_tons: 10,
        }
    } else {
        item.quantity
    };
    let mut label = Vec::from(&b"commerce/buyer-search/v1"[..]);
    label.extend_from_slice(&assignment_id.to_be_bytes());
    let mut random = SeedStream::new(derive_seed(system_seed, &label)?);
    fitting_quantity(&mut random, dice, maximum_quantity_millitons)
}

fn fitting_quantity(
    random: &mut SeedStream,
    dice: QuantityDice,
    maximum_quantity_millitons: u64,
) -> Result<Option<u64>, CryptoError> {
    let minimum = u64::from(dice.dice) * u64::from(dice.multiplier_tons) * MILLITONS_PER_TON;
    if maximum_quantity_millitons != 0 && maximum_quantity_millitons < minimum {
        return Ok(None);
    }
    for _ in 0..256 {
        let quantity = roll_quantity(random, dice)? * MILLITONS_PER_TON;
        if maximum_quantity_millitons == 0 || quantity <= maximum_quantity_millitons {
            return Ok(Some(quantity));
        }
    }
    Ok(Some(minimum))
}

pub fn purchase_price_for_effect(
    item: CommodityDefinition,
    world: &World,
    negotiation_effect: i16,
) -> u64 {
    let codes = world_trade_codes(world);
    let effect =
        negotiation_effect + i16::from(strongest_modifier(&item.purchase_modifiers, &codes));
    percentage(item.base_price_per_ton, purchase_percent(effect))
}

pub fn sale_price_for_effect(
    valuation_basis_per_ton: u64,
    item: CommodityDefinition,
    world: &World,
    negotiation_effect: i16,
) -> u64 {
    let codes = world_trade_codes(world);
    let effect = negotiation_effect + i16::from(strongest_modifier(&item.sale_modifiers, &codes));
    percentage(valuation_basis_per_ton, 100 + sale_markup_percent(effect))
}

pub fn world_trade_codes(world: &World) -> Vec<TradeCode> {
    let mut result = Vec::new();
    if world.is_agricultural() {
        result.push(TradeCode::Agricultural);
    }
    if world.size == 0 && world.atmosphere == 0 && world.hydrographics == 0 {
        result.push(TradeCode::Asteroid);
    }
    if world.atmosphere >= 10 && world.hydrographics >= 1 {
        result.push(TradeCode::FluidOceans);
    }
    if (6..=8).contains(&world.size)
        && matches!(world.atmosphere, 5 | 6 | 8)
        && (5..=7).contains(&world.hydrographics)
    {
        result.push(TradeCode::Garden);
    }
    if world.population >= 9 {
        result.push(TradeCode::HighPopulation);
    }
    if world.tech_level >= 12 {
        result.push(TradeCode::HighTechnology);
    }
    if world.atmosphere <= 1 && world.hydrographics >= 1 {
        result.push(TradeCode::IceCapped);
    }
    if world.is_industrial() {
        result.push(TradeCode::Industrial);
    }
    if world.atmosphere <= 3 && world.hydrographics <= 3 && world.population >= 6 {
        result.push(TradeCode::NonAgricultural);
    }
    if (4..=6).contains(&world.population) {
        result.push(TradeCode::NonIndustrial);
    }
    if (2..=5).contains(&world.atmosphere) && world.hydrographics <= 3 {
        result.push(TradeCode::Poor);
    }
    if matches!(world.atmosphere, 6 | 8) && (6..=8).contains(&world.population) {
        result.push(TradeCode::Rich);
    }
    if world.atmosphere == 0 {
        result.push(TradeCode::Vacuum);
    }
    result
}

pub fn commodity_legality(item: CommodityDefinition, world: &World) -> Legality {
    let threshold: u8 = match item.d66 {
        64 => 3,
        65 => 2,
        34 => 5,
        25 => 6,
        43 => 8,
        22 => 9,
        61 => 7,
        _ => return Legality::Legal,
    };
    if world.law_level >= threshold.saturating_add(3) {
        Legality::Prohibited
    } else if world.law_level >= threshold {
        Legality::Restricted
    } else {
        Legality::Legal
    }
}

pub fn starting_operating_reserve(cargo_capacity_millitons: u64) -> u64 {
    cargo_capacity_millitons.saturating_mul(STARTING_RESERVE_REFERENCE_PRICE_PER_TON)
        / MILLITONS_PER_TON
}

pub fn purchase_cost_credits(price_per_ton: u64, quantity_millitons: u64) -> Option<u64> {
    let numerator = u128::from(price_per_ton) * u128::from(quantity_millitons);
    let credits = numerator.div_ceil(u128::from(MILLITONS_PER_TON));
    credits.try_into().ok()
}

pub fn sale_proceeds_credits(price_per_ton: u64, quantity_millitons: u64) -> Option<u64> {
    let numerator = u128::from(price_per_ton) * u128::from(quantity_millitons);
    let credits = numerator / u128::from(MILLITONS_PER_TON);
    credits.try_into().ok()
}

fn roll_dice(random: &mut SeedStream, dice: u8, sides: u8) -> Result<u64, CryptoError> {
    let mut total = 0;
    for _ in 0..dice {
        total += random.next_u64()? % u64::from(sides) + 1;
    }
    Ok(total)
}

fn roll_quantity(random: &mut SeedStream, dice: QuantityDice) -> Result<u64, CryptoError> {
    Ok(roll_dice(random, dice.dice, dice.sides)? * u64::from(dice.multiplier_tons))
}

fn strongest_modifier(modifiers: &[TradeModifier; 2], codes: &[TradeCode]) -> i8 {
    modifiers
        .iter()
        .filter(|modifier| modifier.dm != 0 && codes.contains(&modifier.code))
        .map(|modifier| modifier.dm)
        .max()
        .unwrap_or(0)
}

fn purchase_percent(effect: i16) -> u64 {
    match effect {
        6.. => 80,
        0..=5 => 90,
        -5..=-1 => 100,
        ..=-6 => 120,
    }
}

fn sale_markup_percent(effect: i16) -> u64 {
    match effect {
        6.. => 30,
        0..=5 => 15,
        -5..=-1 => 2,
        ..=-6 => 0,
    }
}

fn percentage(base: u64, percent: u64) -> u64 {
    base.saturating_mul(percent) / 100
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World {
            id: 1,
            system_id: 1,
            name: "Test".into(),
            starport: Starport::A,
            size: 7,
            atmosphere: 6,
            hydrographics: 6,
            population: 7,
            population_multiplier: 5,
            government: 4,
            law_level: 4,
            tech_level: 13,
            planetoid_belts: 0,
            gas_giants: 1,
        }
    }

    #[test]
    fn revised_catalog_is_complete_and_has_no_unusual_placeholder() {
        assert_eq!(COMMON_GOODS.len() + TRADE_GOODS.len(), 41);
        assert_eq!(TRADE_GOODS.first().unwrap().d66, 11);
        assert_eq!(TRADE_GOODS.last().unwrap().d66, 65);
        assert!(TRADE_GOODS.iter().all(|item| item.d66 != 66));
        assert!(
            TRADE_GOODS
                .windows(2)
                .all(|items| items[0].id < items[1].id)
        );
    }

    #[test]
    fn world_trade_codes_follow_ce_uwp_predicates() {
        let codes = world_trade_codes(&world());
        assert!(codes.contains(&TradeCode::Agricultural));
        assert!(codes.contains(&TradeCode::Garden));
        assert!(codes.contains(&TradeCode::Rich));
        assert!(codes.contains(&TradeCode::HighTechnology));
        assert!(!codes.contains(&TradeCode::Industrial));
    }

    #[test]
    fn supplier_search_uses_source_outcomes_and_respects_the_stated_maximum() {
        let failed = supplier_lots([0x42; 32], 31, -1, u64::MAX, &world()).unwrap();
        let succeeded = supplier_lots([0x42; 32], 31, 0, u64::MAX, &world()).unwrap();
        let spectacular = supplier_lots([0x42; 32], 31, 6, u64::MAX, &world()).unwrap();
        assert!(!failed.is_empty());
        assert!(failed.iter().all(|lot| lot.commodity.common));
        assert!(succeeded.iter().any(|lot| !lot.commodity.common));
        assert_eq!(spectacular.len(), succeeded.len() * 2);
        assert!(
            supplier_lots([0x42; 32], 31, -6, u64::MAX, &world())
                .unwrap()
                .is_empty()
        );

        let bounded = supplier_lots([0x42; 32], 31, 0, 20 * MILLITONS_PER_TON, &world()).unwrap();
        assert!(!bounded.is_empty());
        assert!(
            bounded
                .iter()
                .all(|lot| lot.quantity_millitons <= 20 * MILLITONS_PER_TON)
        );
    }

    #[test]
    fn buyer_lot_size_and_negotiated_prices_use_the_published_boundaries() {
        let item = commodity(1).unwrap();
        assert_eq!(
            buyer_lot_quantity([0x42; 32], 31, item, MILLITONS_PER_TON, &world(),).unwrap(),
            None
        );
        assert_eq!(
            buyer_lot_quantity([0x42; 32], 31, item, 20 * MILLITONS_PER_TON, &world(),).unwrap(),
            Some(20 * MILLITONS_PER_TON)
        );
        assert_eq!(purchase_price_for_effect(item, &world(), 6), 800);
        assert_eq!(purchase_price_for_effect(item, &world(), -6), 1_200);
        assert_eq!(sale_price_for_effect(7_500, item, &world(), 6), 9_750);
        assert_eq!(sale_price_for_effect(7_500, item, &world(), -6), 7_500);
    }

    #[test]
    fn restricted_goods_derive_from_law_level() {
        let weapons = commodity(45).unwrap();
        assert_eq!(commodity_legality(weapons, &world()), Legality::Restricted);
        let mut strict = world();
        strict.law_level = 9;
        assert_eq!(commodity_legality(weapons, &strict), Legality::Prohibited);
    }

    #[test]
    fn reserve_fills_the_hold_with_reference_common_goods() {
        assert_eq!(starting_operating_reserve(92_000), 1_840_000);
        assert_eq!(starting_operating_reserve(44_500), 890_000);
    }

    #[test]
    fn fractional_cargo_prices_favour_the_market_at_the_credit_boundary() {
        assert_eq!(purchase_cost_credits(1_100, 1), Some(2));
        assert_eq!(sale_proceeds_credits(1_100, 1), Some(1));
        assert_eq!(purchase_cost_credits(25_000, 1_001), Some(25_025));
        assert_eq!(sale_proceeds_credits(25_000, 1_001), Some(25_025));
    }
}
