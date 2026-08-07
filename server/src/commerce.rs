//! Cepheus Trader's authoritative merchant rules.
//!
//! Generic commodity data and availability come from the revised Open Game
//! Content table in *Bounded Fortune*.  Proper names from its optional colour
//! tables are intentionally absent.  Negotiation uses the Clement task
//! outcomes instead of the core CE extreme-percentage table.

use std::collections::BTreeMap;

use crate::crypto::{CryptoError, SeedStream, derive_seed};
use crate::universe::{Starport, World};
use crate::wire::PlayerIdentity;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketQuote {
    pub offer_id: u64,
    pub commodity: CommodityDefinition,
    pub available_millitons: u64,
    pub purchase_price_per_ton: u64,
    pub indicative_sale_price_per_ton: u64,
    pub legality: Legality,
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

pub fn quote_market_goods(
    system_seed: [u8; 32],
    system_id: u64,
    game_day: u64,
    identity: &PlayerIdentity,
    broker_level: i8,
    charisma: u8,
    world: &World,
) -> Result<Vec<MarketQuote>, CryptoError> {
    let mut stock_label = Vec::with_capacity(48);
    stock_label.extend_from_slice(b"commerce/market-stock/v1");
    stock_label.extend_from_slice(&game_day.to_be_bytes());
    let mut stock_random = SeedStream::new(derive_seed(system_seed, &stock_label)?);
    let mut price_label = Vec::with_capacity(64);
    price_label.extend_from_slice(b"commerce/market-price/v1");
    price_label.extend_from_slice(&game_day.to_be_bytes());
    price_label.extend_from_slice(&identity.bbs_id.to_be_bytes());
    price_label.extend_from_slice(&identity.player_id.to_be_bytes());
    let mut price_random = SeedStream::new(derive_seed(system_seed, &price_label)?);
    let codes = world_trade_codes(world);
    let mut stock: BTreeMap<u16, u64> = BTreeMap::new();

    let (common_lot_dice, common_size_dice, trade_lot_dice) = match world.starport {
        Starport::A => (6, 2, 4),
        Starport::B => (4, 1, 3),
        Starport::C => (2, 1, 2),
        Starport::D | Starport::E => (1, 1, 1),
        Starport::X => (0, 0, 0),
    };
    let common_lots = roll_dice(&mut stock_random, common_lot_dice, 6)?;
    for _ in 0..common_lots {
        let item = COMMON_GOODS[(stock_random.next_u64()? % 6) as usize];
        let size = roll_dice(&mut stock_random, common_size_dice, 6)? * 10;
        *stock.entry(item.id).or_default() += size * MILLITONS_PER_TON;
    }
    let trade_lots = roll_dice(&mut stock_random, trade_lot_dice, 6)?;
    for _ in 0..trade_lots {
        let item = TRADE_GOODS[(stock_random.next_u64()? % TRADE_GOODS.len() as u64) as usize];
        let size = roll_quantity(&mut stock_random, item.quantity)?;
        *stock.entry(item.id).or_default() += size * MILLITONS_PER_TON;
    }

    let skill_dm = i16::from(broker_level) + i16::from(characteristic_dm(charisma));
    stock
        .into_iter()
        .map(|(commodity_id, available_millitons)| {
            let item = commodity(commodity_id).expect("generated commodity exists");
            let purchase_dm = strongest_modifier(&item.purchase_modifiers, &codes);
            let sale_dm = strongest_modifier(&item.sale_modifiers, &codes);
            let purchase_effect =
                task_effect(&mut price_random, skill_dm + i16::from(purchase_dm), -2)?;
            let sale_effect = task_effect(&mut price_random, skill_dm + i16::from(sale_dm), -2)?;
            let purchase_price =
                percentage(item.base_price_per_ton, purchase_percent(purchase_effect));
            let indicative_sale =
                percentage(purchase_price, 100 + sale_markup_percent(sale_effect));
            Ok(MarketQuote {
                offer_id: market_offer_id(system_id, game_day, item.id),
                commodity: item,
                available_millitons,
                purchase_price_per_ton: purchase_price,
                indicative_sale_price_per_ton: indicative_sale,
                legality: commodity_legality(item, world),
            })
        })
        .collect()
}

pub fn negotiated_sale_price(
    system_seed: [u8; 32],
    game_day: u64,
    identity: &PlayerIdentity,
    cargo_lot_id: u64,
    purchase_price_per_ton: u64,
    broker_level: i8,
    charisma: u8,
    item: CommodityDefinition,
    world: &World,
) -> Result<u64, CryptoError> {
    let mut label = Vec::new();
    label.extend_from_slice(b"commerce/sale-negotiation/v1");
    label.extend_from_slice(&game_day.to_be_bytes());
    label.extend_from_slice(&identity.bbs_id.to_be_bytes());
    label.extend_from_slice(&identity.player_id.to_be_bytes());
    label.extend_from_slice(&cargo_lot_id.to_be_bytes());
    let mut random = SeedStream::new(derive_seed(system_seed, &label)?);
    let codes = world_trade_codes(world);
    let dm = i16::from(broker_level)
        + i16::from(characteristic_dm(charisma))
        + i16::from(strongest_modifier(&item.sale_modifiers, &codes));
    let effect = task_effect(&mut random, dm, -2)?;
    Ok(percentage(
        purchase_price_per_ton,
        100 + sale_markup_percent(effect),
    ))
}

pub fn market_offer_id(system_id: u64, game_day: u64, commodity_id: u16) -> u64 {
    system_id.rotate_left(17) ^ game_day.rotate_left(7) ^ u64::from(commodity_id)
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

fn characteristic_dm(score: u8) -> i8 {
    match score {
        0 => -3,
        1..=2 => -2,
        3..=5 => -1,
        6..=8 => 0,
        9..=11 => 1,
        12..=14 => 2,
        _ => 3,
    }
}

fn strongest_modifier(modifiers: &[TradeModifier; 2], codes: &[TradeCode]) -> i8 {
    modifiers
        .iter()
        .filter(|modifier| modifier.dm != 0 && codes.contains(&modifier.code))
        .map(|modifier| modifier.dm)
        .max()
        .unwrap_or(0)
}

fn task_effect(random: &mut SeedStream, dm: i16, difficulty: i16) -> Result<i16, CryptoError> {
    Ok(roll_dice(random, 2, 6)? as i16 + dm + difficulty - 8)
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
    fn market_is_repeatable_and_uses_bounded_price_outcomes() {
        let identity = PlayerIdentity {
            bbs_id: 7,
            player_id: 9,
        };
        let first = quote_market_goods([0x42; 32], 31, 4, &identity, 2, 9, &world()).unwrap();
        let second = quote_market_goods([0x42; 32], 31, 4, &identity, 2, 9, &world()).unwrap();
        assert_eq!(first, second);
        assert!(!first.is_empty());
        assert!(first.iter().all(|line| {
            [80, 90, 100, 120]
                .contains(&(line.purchase_price_per_ton * 100 / line.commodity.base_price_per_ton))
                && line.available_millitons % MILLITONS_PER_TON == 0
        }));
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
