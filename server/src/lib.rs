//! Authoritative Cepheus Trader server.

// Domain transactions deliberately keep their full argument lists and
// explicit nested guards visible. Mutable LMDB transaction reborrows are also
// written out to make their lifetime boundaries obvious during review.
#![allow(
    clippy::collapsible_if,
    clippy::filter_map_bool_then,
    clippy::manual_checked_ops,
    clippy::manual_is_multiple_of,
    clippy::needless_borrow,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_filter_map
)]

pub mod ct_rpc_capnp {
    include!(concat!(env!("OUT_DIR"), "/ct_rpc_capnp.rs"));
}

pub mod ct_admin_capnp {
    include!(concat!(env!("OUT_DIR"), "/ct_admin_capnp.rs"));
}

pub mod ct_sysop_capnp {
    include!(concat!(env!("OUT_DIR"), "/ct_sysop_capnp.rs"));
}

pub mod admin_psk;
pub mod admin_wire;
pub mod bbs_polity;
pub mod careers;
pub mod celestial;
pub mod clock;
pub mod combat;
pub mod commerce;
pub mod coverage;
pub mod cpu_time;
pub mod creation;
pub mod crypto;
pub mod engine;
pub mod i18n;
pub mod jump;
pub mod navigation;
pub mod person_names;
pub mod personnel;
pub mod place_names;
pub mod server;
pub mod ship_condition;
pub mod simulation;
pub mod store;
pub mod sysop_wire;
pub mod task_resolution;
pub mod tls;
pub mod traffic;
pub mod universe;
pub mod wire;
