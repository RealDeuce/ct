// This build script belongs only to the Rust server project.
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../protocol/ct_rpc.capnp");
    println!("cargo:rerun-if-changed=../protocol/ct_admin.capnp");
    println!("cargo:rerun-if-changed=../protocol/ct_sysop.capnp");
    println!("cargo:rerun-if-changed=native/ct_gnutls.c");
    println!("cargo:rerun-if-changed=native/ct_gnutls.h");
    println!("cargo:rerun-if-changed=native/ct_clock.c");
    println!("cargo:rerun-if-changed=native/ct_clock.h");
    println!("cargo:rerun-if-changed=../catalog/ships");
    println!("cargo:rerun-if-changed=../catalog/traffic-names.toml");
    println!("cargo:rerun-if-changed=../catalog/person-names.toml");
    println!("cargo:rerun-if-changed=../catalog/place-names.toml");
    capnpc::CompilerCommand::new()
        .src_prefix("../protocol")
        .file("../protocol/ct_rpc.capnp")
        .file("../protocol/ct_admin.capnp")
        .file("../protocol/ct_sysop.capnp")
        .run()
        .expect("compile shared protocol/ct_rpc.capnp");

    let gnutls = pkg_config::Config::new()
        .atleast_version("3.8")
        .probe("gnutls")
        .expect("GnuTLS 3.8 or newer is required");
    let mut native = cc::Build::new();
    native.file("native/ct_gnutls.c").file("native/ct_clock.c");
    for include in gnutls.include_paths {
        native.include(include);
    }
    native.warnings_into_errors(true).compile("ct_gnutls");

    generate_traffic_catalog();
    generate_person_name_catalog();
    generate_place_name_catalog();
    generate_ship_source_catalog();
}

fn generate_person_name_catalog() {
    let source =
        fs::read_to_string("../catalog/person-names.toml").expect("read personnel naming catalog");
    assert_eq!(
        scalar(&source, "schema_version"),
        Some(1),
        "unsupported personnel naming schema"
    );
    let revision = scalar(&source, "catalog_revision").expect("personnel catalog revision");
    let given = text_array(&source, "given_names").expect("personnel given_names");
    let family = text_array(&source, "family_names").expect("personnel family_names");
    assert!(
        given.len() >= 64,
        "personnel catalog needs at least 64 given names"
    );
    assert!(
        family.len() >= 64,
        "personnel catalog needs at least 64 family names"
    );
    let mut unique = std::collections::BTreeSet::new();
    for name in given.iter().chain(&family) {
        assert!(
            !name.is_empty()
                && name.is_ascii()
                && name.bytes().all(|byte| !byte.is_ascii_control()),
            "personnel names must be nonempty ISO 646 text"
        );
        assert!(
            unique.insert(name.to_ascii_lowercase()),
            "duplicate personnel name component {name}"
        );
    }
    let mut generated = format!("pub const PERSON_NAME_CATALOG_REVISION: u64 = {revision};\n");
    generated.push_str("pub static PERSON_GIVEN_NAMES: &[&str] = ");
    push_string_slice(&mut generated, &given);
    generated.push_str(";\npub static PERSON_FAMILY_NAMES: &[&str] = ");
    push_string_slice(&mut generated, &family);
    generated.push_str(";\n");
    let output = std::env::var_os("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&output).join("person_name_catalog.rs"), generated)
        .expect("write generated personnel naming catalog");
}

fn generate_ship_source_catalog() {
    let directory = Path::new("../catalog/ships");
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory).expect("read ship catalog") {
        let path = entry.expect("ship catalog entry").path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("ship-") || !name.ends_with(".toml") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("read ship catalog record");
        let id = scalar(&source, "catalog_id").expect("ship catalog_id");
        let absolute = fs::canonicalize(&path).expect("canonical ship catalog path");
        entries.push((id, absolute));
    }
    entries.sort_by_key(|entry| entry.0);
    for pair in entries.windows(2) {
        assert_ne!(pair[0].0, pair[1].0, "duplicate ship catalog ID");
    }
    let mut generated = String::from("static SHIP_SOURCES: &[(u32, &str)] = &[\n");
    for (id, path) in entries {
        generated.push_str(&format!(
            "    ({id}, include_str!({:?})),\n",
            path.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    let runtime_source =
        fs::read_to_string("../catalog/ship-runtime.toml").expect("read ship runtime catalog");
    generated.push_str("static SHIP_RUNTIME_COMPONENTS: &[RuntimeComponent] = &[\n");
    for section in runtime_source.split("[[component]]").skip(1) {
        let catalog_id = scalar(section, "catalog_id").expect("runtime component catalog_id");
        let kind = text(section, "kind").expect("runtime component kind");
        let component_id = text(section, "component_id").expect("runtime component ID");
        let quantity = scalar(section, "quantity").expect("runtime component quantity");
        let displacement =
            scalar(section, "displacement_millitons").expect("runtime component displacement");
        let price = scalar(section, "price_credits").expect("runtime component price");
        let pack_units = scalar(section, "pack_units").expect("runtime component pack units");
        generated.push_str(&format!(
            "    RuntimeComponent {{ catalog_id: {catalog_id}, kind: {kind:?}, component_id: {component_id:?}, quantity: {quantity}, displacement_millitons: {displacement}, price_credits: {price}, pack_units: {pack_units} }},\n"
        ));
    }
    generated.push_str("];\n");
    assert_eq!(
        scalar(&runtime_source, "schema_version"),
        Some(1),
        "unsupported ship runtime catalog schema"
    );
    let ship_index =
        fs::read_to_string("../catalog/ships/index.toml").expect("read authoritative ship index");
    assert_eq!(
        scalar(&runtime_source, "catalog_revision"),
        scalar(&ship_index, "catalog_revision"),
        "ship runtime catalog is stale; regenerate it with the validator"
    );
    generated.push_str("static SHIP_RUNTIME: &[RuntimeShip] = &[\n");
    for section in runtime_source.split("[[ship]]").skip(1) {
        let id = scalar(section, "catalog_id").expect("runtime ship catalog_id");
        let name = text(section, "display_name").expect("runtime ship display_name");
        let tl = scalar(section, "tech_level").expect("runtime ship tech_level");
        let price = scalar(section, "construction_price_credits").expect("runtime ship price");
        let displacement =
            scalar(section, "displacement_millitons").expect("runtime ship displacement");
        let jump = scalar(section, "jump_rating").expect("runtime ship jump rating");
        let thrust = scalar(section, "thrust_g").expect("runtime ship thrust");
        let fuel = scalar(section, "fuel_millitons").expect("runtime ship fuel");
        let jump_fuel = scalar(section, "jump_fuel_millitons").expect("runtime ship jump fuel");
        let cargo = scalar(section, "cargo_millitons").expect("runtime ship cargo");
        let crew = scalar(section, "minimum_crew").expect("runtime ship crew");
        generated.push_str(&format!(
            "    RuntimeShip {{ catalog_id: {id}, class_name: {name:?}, tech_level: {tl}, price_credits: {price}, displacement_millitons: {displacement}, jump_rating: {jump}, thrust_g: {thrust}, fuel_millitons: {fuel}, jump_fuel_millitons: {jump_fuel}, cargo_millitons: {cargo}, minimum_crew: {crew} }},\n"
        ));
    }
    generated.push_str("];\n");
    let output = std::env::var_os("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&output).join("ship_source_catalog.rs"), generated)
        .expect("write generated ship source catalog");
}

fn generate_place_name_catalog() {
    let source =
        fs::read_to_string("../catalog/place-names.toml").expect("read place naming catalog");
    let schema = scalar(&source, "schema_version").expect("place naming schema_version");
    assert_eq!(schema, 1, "unsupported place naming schema");
    let mut profiles = Vec::new();
    for section in source.split("[[profile]]").skip(1) {
        let id = scalar(section, "id").expect("place naming profile id");
        let tag = text(section, "tag").expect("place naming profile tag");
        let system_pattern = text(section, "system_pattern").expect("place naming system_pattern");
        let world_pattern = text(section, "world_pattern").expect("place naming world_pattern");
        let mut arrays = Vec::new();
        for key in [
            "system_a", "system_b", "system_c", "system_d", "world_a", "world_b", "world_c",
            "world_d",
        ] {
            let values = text_array_preserving_empty(section, key)
                .unwrap_or_else(|| panic!("place naming {key}"));
            assert!(!values.is_empty(), "place naming {key} must not be empty");
            arrays.push(values);
        }
        for token in ["{0}", "{1}", "{2}", "{3}"] {
            assert!(
                system_pattern.contains(token),
                "system pattern for {tag} omits {token}"
            );
            assert!(
                world_pattern.contains(token),
                "world pattern for {tag} omits {token}"
            );
        }
        profiles.push((id, tag, system_pattern, world_pattern, arrays));
    }
    profiles.sort_by_key(|entry| entry.0);
    assert!(
        profiles.len() >= 2,
        "at least two naming profiles are required"
    );
    let mut generated = String::from("pub static PLACE_NAME_PROFILES: &[PlaceNameProfile] = &[\n");
    for (expected, (id, tag, system_pattern, world_pattern, arrays)) in (1_u64..).zip(&profiles) {
        assert_eq!(expected, *id, "place naming profile IDs must be contiguous");
        generated.push_str(&format!(
            "    PlaceNameProfile {{ id: {id}, tag: {tag:?}, system_pattern: {system_pattern:?}, world_pattern: {world_pattern:?}, system_parts: ["
        ));
        for values in &arrays[..4] {
            push_string_slice(&mut generated, values);
            generated.push_str(", ");
        }
        generated.push_str("], world_parts: [");
        for values in &arrays[4..] {
            push_string_slice(&mut generated, values);
            generated.push_str(", ");
        }
        generated.push_str("] },\n");
    }
    generated.push_str("];\n");
    let output = std::env::var_os("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&output).join("place_name_catalog.rs"), generated)
        .expect("write generated place naming catalog");
}

fn push_string_slice(output: &mut String, values: &[String]) {
    output.push_str("&[");
    for value in values {
        output.push_str(&format!("{value:?}, "));
    }
    output.push(']');
}

fn generate_traffic_catalog() {
    let directory = Path::new("../catalog/ships");
    let mut designs = Vec::new();
    for entry in fs::read_dir(directory).expect("read ship catalog") {
        let path = entry.expect("ship catalog entry").path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("ship-") || !name.ends_with(".toml") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("read ship catalog record");
        let Some(catalog_id) = scalar(&source, "catalog_id") else {
            continue;
        };
        let display_name =
            text(&source, "display_name").unwrap_or_else(|| format!("Ship {catalog_id}"));
        let primary_role =
            text(&source, "primary_role").unwrap_or_else(|| "general-purpose".into());
        let vessel_kind = text(&source, "vessel_kind").unwrap_or_default();
        let path_id = scalar(&source, "upgrade_path_id").unwrap_or(1);
        let tech_level = scalar(&source, "tech_level").unwrap_or(0);
        let jump_rating = scalar(&source, "jump_rating")
            .or_else(|| scalar(&source, "jump_distance"))
            .unwrap_or(0);
        let displacement = scalar(&source, "effective_displacement_millitons")
            .or_else(|| scalar(&source, "accounted_displacement_millitons"))
            .or_else(|| scalar(&source, "hull_millitons"))
            .unwrap_or(0);
        if vessel_kind != "starship" || jump_rating == 0 || displacement == 0 {
            continue;
        }
        designs.push((
            catalog_id,
            display_name,
            primary_role,
            path_id,
            tech_level,
            jump_rating,
            displacement,
        ));
    }
    designs.sort_by_key(|entry| entry.0);
    let mut generated = String::from("pub static TRAFFIC_DESIGNS: &[TrafficDesign] = &[\n");
    for (id, name, role, path, tl, jump, displacement) in designs {
        generated.push_str(&format!(
            "    TrafficDesign {{ catalog_id: {id}, class_name: {name:?}, role: {role:?}, path_id: {path}, tech_level: {tl}, jump_rating: {jump}, displacement_millitons: {displacement} }},\n"
        ));
    }
    generated.push_str("];\n");
    let names =
        fs::read_to_string("../catalog/traffic-names.toml").expect("read traffic naming catalog");
    let mut operators = Vec::new();
    let mut pools = Vec::new();
    for section in names.split("[[path]]").skip(1) {
        let path_id = scalar(section, "path_id").expect("traffic path_id");
        let operator = text(section, "operator_label").expect("traffic operator_label");
        let pool = text_array(section, "ship_names").expect("traffic ship_names");
        operators.push((path_id, operator));
        pools.push((path_id, pool));
    }
    operators.sort_by_key(|entry| entry.0);
    pools.sort_by_key(|entry| entry.0);
    assert_eq!(
        operators.len(),
        9,
        "traffic naming catalog must define nine paths"
    );
    assert_eq!(
        pools.len(),
        9,
        "traffic naming catalog must define nine paths"
    );
    generated.push_str("pub static TRAFFIC_OPERATORS: &[&str] = &[\n");
    for (expected, (path_id, operator)) in (1_u64..=9).zip(&operators) {
        assert_eq!(
            expected, *path_id,
            "traffic path identifiers must be contiguous"
        );
        generated.push_str(&format!("    {operator:?},\n"));
    }
    generated.push_str("];\n");
    generated.push_str("pub static TRAFFIC_NAMES: &[&[&str]] = &[\n");
    for (expected, (path_id, pool)) in (1_u64..=9).zip(&pools) {
        assert_eq!(
            expected, *path_id,
            "traffic path identifiers must be contiguous"
        );
        assert!(!pool.is_empty(), "traffic name pools must not be empty");
        generated.push_str("    &[");
        for name in pool {
            generated.push_str(&format!("{name:?}, "));
        }
        generated.push_str("],\n");
    }
    generated.push_str("];\n");
    let output = std::env::var_os("OUT_DIR").expect("OUT_DIR");
    fs::write(Path::new(&output).join("traffic_catalog.rs"), generated)
        .expect("write generated traffic catalog");
}

fn scalar(source: &str, key: &str) -> Option<u64> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn text(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
    })
}

fn text_array(source: &str, key: &str) -> Option<Vec<String>> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        if candidate.trim() != key {
            return None;
        }
        let value = value.trim().strip_prefix('[')?.strip_suffix(']')?;
        Some(
            value
                .split(',')
                .map(|item| item.trim().trim_matches('"').to_owned())
                .filter(|item| !item.is_empty())
                .collect(),
        )
    })
}

fn text_array_preserving_empty(source: &str, key: &str) -> Option<Vec<String>> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        if candidate.trim() != key {
            return None;
        }
        let value = value.trim().strip_prefix('[')?.strip_suffix(']')?;
        Some(
            value
                .split(',')
                .map(|item| item.trim().trim_matches('"').to_owned())
                .collect(),
        )
    })
}
