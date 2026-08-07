"""Evaluate CE small craft against the core and small-craft rule modules."""

from __future__ import annotations

from typing import Any

from ship_design import (
    DesignError,
    Totals,
    _check_keys,
    _index,
    _integer,
    _records,
    _require_tl,
    _table,
    _text,
    _text_list,
)


def evaluate_small_craft(
    core: dict[str, Any],
    small: dict[str, Any],
    design: dict[str, Any],
    extension: dict[str, Any] | None = None,
    component_rules: dict[str, Any] | None = None,
) -> dict[str, Any]:
    if small.get("schema_version") != 1:
        raise DesignError("small-craft rules schema_version must be 1")
    ruleset_id = _text(
        (extension or small).get("ruleset_id"),
        "small ruleset_id",
    )
    if design.get("ruleset_id") != ruleset_id:
        raise DesignError(f"design must select ruleset_id {ruleset_id!r}")
    if small.get("extends_ruleset_id") != core.get("ruleset_id"):
        raise DesignError("small-craft rules do not extend the selected core rules")
    if extension is not None:
        if extension.get("schema_version") != 1:
            raise DesignError("small-craft extension schema_version must be 1")
        if extension.get("extends_ruleset_id") != small.get("ruleset_id"):
            raise DesignError("small-craft extension selects the wrong base")
    _check_keys(
        design,
        {
            "schema_version",
            "design_id",
            "revision",
            "ruleset_id",
            "source_ids",
            "catalog",
            "tech_level",
            "standard_design",
            "hull",
            "armor",
            "hull_options",
            "drives",
            "fuel",
            "control",
            "computer",
            "software",
            "electronics",
            "equipment",
            "parameterized_equipment",
            "airlocks",
            "additional_fire_control_stations",
            "unused_fire_control_stations",
            "mounts",
            "ammunition",
            "crew",
            "cargo_millitons",
            "assertions",
        },
        "design",
    )
    if design.get("schema_version") != 1:
        raise DesignError("design schema_version must be 1")
    _text(design.get("design_id"), "design.design_id")
    _integer(design.get("revision"), "design.revision", 1)
    design_tl = _integer(design.get("tech_level"), "design.tech_level", 1)
    standard = design.get("standard_design")
    if not isinstance(standard, bool):
        raise DesignError("design.standard_design must be boolean")
    required_sources = set(
        _text_list(core.get("source_ids"), "core.source_ids", 1)
    ) | set(_text_list(small.get("source_ids"), "small.source_ids", 1))
    if extension is not None:
        required_sources |= set(
            _text_list(extension.get("source_ids"), "extension.source_ids", 1)
        )
    design_sources = set(
        _text_list(design.get("source_ids"), "design.source_ids", 1)
    )
    if not required_sources.issubset(design_sources):
        raise DesignError("design.source_ids omits a construction-rules source")

    hulls = _index(small.get("hull"), "id", "small.hull")
    configurations = _index(
        core.get("configuration"), "id", "core.configuration"
    )
    armors = _index(core.get("armor"), "id", "core.armor")
    armor_extensions: dict[str, dict[str, Any]] = {}
    if extension is not None:
        armor_extensions = _index(
            extension.get("armor_extension"),
            "id",
            "extension.armor_extension",
        )
    hull_options = _index(core.get("hull_option"), "id", "core.hull_option")
    if extension is not None:
        extension_hull_options = _index(
            extension.get("hull_option"), "id", "extension.hull_option"
        )
        overlap = set(hull_options).intersection(extension_hull_options)
        if overlap:
            raise DesignError(
                f"small-craft extension duplicates hull option "
                f"{sorted(overlap)[0]!r}"
            )
        hull_options.update(extension_hull_options)
    drives = _index(small.get("drive"), "code", "small.drive")
    controls = _index(small.get("control"), "id", "small.control")
    if extension is not None:
        extension_controls = _index(
            extension.get("control"), "id", "extension.control"
        )
        overlap = set(controls).intersection(extension_controls)
        if overlap:
            raise DesignError(
                f"small-craft extension duplicates control {sorted(overlap)[0]!r}"
            )
        controls.update(extension_controls)
    computers = _index(core.get("computer"), "id", "core.computer")
    computer_options = _index(
        core.get("computer_option"), "id", "core.computer_option"
    )
    software_rules = _index(core.get("software"), "id", "core.software")
    electronics_rules = _index(
        core.get("electronics"), "id", "core.electronics"
    )
    component_source = component_rules or core
    equipment_rules = _index(
        component_source.get("equipment"),
        "id",
        "component_rules.equipment",
    )
    if extension is not None:
        extension_equipment = _index(
            extension.get("equipment"), "id", "extension.equipment"
        )
        overlap = set(equipment_rules).intersection(extension_equipment)
        if overlap:
            raise DesignError(
                f"small-craft extension duplicates equipment {sorted(overlap)[0]!r}"
            )
        equipment_rules.update(extension_equipment)
    parameterized_equipment_rules = _index(
        component_source.get("parameterized_equipment"),
        "id",
        "component_rules.parameterized_equipment",
    )
    if extension is not None:
        extension_parameterized = _index(
            extension.get("parameterized_equipment"),
            "id",
            "extension.parameterized_equipment",
        )
        overlap = set(parameterized_equipment_rules).intersection(
            extension_parameterized
        )
        if overlap:
            raise DesignError(
                "small-craft extension duplicates parameterized equipment "
                f"{sorted(overlap)[0]!r}"
            )
        parameterized_equipment_rules.update(extension_parameterized)
    mount_rules = _index(core.get("mount"), "id", "core.mount")
    weapon_rules = _index(core.get("weapon"), "id", "core.weapon")
    ammunition_rules = _index(
        core.get("ammunition"), "id", "core.ammunition"
    )

    hull_choice = _table(design.get("hull"), "design.hull")
    _check_keys(hull_choice, {"id", "configuration"}, "design.hull")
    hull_id = _text(hull_choice.get("id"), "design.hull.id")
    configuration_id = _text(
        hull_choice.get("configuration"), "design.hull.configuration"
    )
    try:
        hull = hulls[hull_id]
        configuration = configurations[configuration_id]
    except KeyError as error:
        raise DesignError(f"unknown small-craft hull/configuration: {error}") from error
    hull_tons = _integer(hull.get("tons"), f"small.hull.{hull_id}.tons", 1)
    hull_millitons = hull_tons * 1000
    base_hull_price = _integer(
        hull.get("price_credits"), f"small.hull.{hull_id}.price_credits"
    )
    configuration_percent = _integer(
        configuration.get("hull_price_percent"),
        f"core.configuration.{configuration_id}.hull_price_percent",
    )
    hull_price = base_hull_price * configuration_percent // 100
    totals = Totals()
    totals.add(0, hull_price)
    line_items: list[dict[str, Any]] = [
        {
            "kind": "small-craft-hull",
            "id": hull_id,
            "quantity": 1,
            "displacement_millitons": 0,
            "price_credits": hull_price,
        }
    ]

    armor_points = 0
    if design.get("armor") is not None:
        choice = _table(design["armor"], "design.armor")
        _check_keys(choice, {"id", "layers", "points"}, "design.armor")
        armor_id = _text(choice.get("id"), "design.armor.id")
        has_layers = "layers" in choice
        has_points = "points" in choice
        if has_layers == has_points:
            raise DesignError("design.armor selects exactly one of layers or points")
        if has_layers:
            quantity = _integer(
                choice.get("layers"), "design.armor.layers", 1
            )
            try:
                armor = armors[armor_id]
            except KeyError as error:
                raise DesignError(f"unknown armor {armor_id!r}") from error
            _require_tl(design_tl, armor, f"armor {armor_id}")
            armor_points = quantity * _integer(
                armor.get("protection_per_layer"),
                f"core.armor.{armor_id}.protection_per_layer",
            )
            maximum = {
                "titanium-steel": min(design_tl, 9),
                "crystaliron": min(design_tl, 13),
                "bonded-superdense": design_tl,
            }[armor_id]
            per_layer_volume = max(
                _integer(
                    armor.get("minimum_millitons_per_layer"),
                    f"core.armor.{armor_id}.minimum_millitons_per_layer",
                ),
                hull_millitons
                * _integer(
                    armor.get("volume_percent_per_layer"),
                    f"core.armor.{armor_id}.volume_percent_per_layer",
                )
                // 100,
            )
            volume = per_layer_volume * quantity
            price = (
                base_hull_price
                * _integer(
                    armor.get("base_hull_price_percent_per_layer"),
                    f"core.armor.{armor_id}.base_hull_price_percent_per_layer",
                )
                * quantity
                // 100
            )
        else:
            quantity = _integer(
                choice.get("points"), "design.armor.points", 1
            )
            try:
                armor_extension = armor_extensions[armor_id]
            except KeyError as error:
                raise DesignError(
                    f"unknown whole-point armor {armor_id!r}"
                ) from error
            base_id = _text(
                armor_extension.get("base_armor_id"),
                f"extension.armor_extension.{armor_id}.base_armor_id",
            )
            try:
                armor = armors[base_id]
            except KeyError as error:
                raise DesignError(
                    f"whole-point armor {armor_id!r} has unknown base "
                    f"{base_id!r}"
                ) from error
            _require_tl(design_tl, armor, f"armor {armor_id}")
            armor_points = quantity
            maximum = {
                "titanium-steel": min(design_tl, 9),
                "crystaliron": min(design_tl, 13),
                "bonded-superdense": design_tl,
            }[base_id]
            volume = (
                hull_millitons
                * _integer(
                    armor_extension.get("volume_basis_points_per_point"),
                    f"extension.armor_extension.{armor_id}."
                    "volume_basis_points_per_point",
                    1,
                )
                * quantity
                + 9999
            ) // 10000
            price = (
                base_hull_price
                * _integer(
                    armor_extension.get(
                        "base_hull_price_basis_points_per_point"
                    ),
                    f"extension.armor_extension.{armor_id}."
                    "base_hull_price_basis_points_per_point",
                    1,
                )
                * quantity
                // 10000
            )
        if armor_points > maximum:
            raise DesignError(
                f"{armor_id} armor {armor_points} exceeds small-craft maximum {maximum}"
            )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "armor",
                "id": armor_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    for position, choice in enumerate(
        _records(design.get("hull_options"), "design.hull_options"), 1
    ):
        label = f"design.hull_options[{position}]"
        _check_keys(choice, {"id"}, label)
        option_id = _text(choice.get("id"), f"{label}.id")
        try:
            option = hull_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown hull option {option_id!r}") from error
        _require_tl(design_tl, option, f"hull option {option_id}")
        price = hull_tons * _integer(
            option.get("price_credits_per_hull_ton"),
            f"core.hull_option.{option_id}.price_credits_per_hull_ton",
        )
        totals.add(0, price)
        line_items.append(
            {
                "kind": "hull-option",
                "id": option_id,
                "quantity": 1,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )

    performance: dict[int, dict[str, int]] = {}
    for position, row in enumerate(
        _records(small.get("drive_performance"), "small.drive_performance"), 1
    ):
        row_tons = _integer(
            row.get("hull_tons"),
            f"small.drive_performance[{position}].hull_tons",
            1,
        )
        performance[row_tons] = {
            code: _integer(value, f"small performance {row_tons}/{code}", 1)
            for code, value in _table(
                row.get("values"),
                f"small.drive_performance[{position}].values",
            ).items()
        }
    drive_choice = _table(design.get("drives"), "design.drives")
    _check_keys(drive_choice, {"maneuver", "power"}, "design.drives")
    maneuver_code = _text(drive_choice.get("maneuver"), "design.drives.maneuver")
    power_code = _text(drive_choice.get("power"), "design.drives.power")
    try:
        maneuver = drives[maneuver_code]
        power = drives[power_code]
        thrust = performance[hull_tons][maneuver_code]
    except KeyError as error:
        raise DesignError(f"invalid small-craft drive selection: {error}") from error
    drive_order = list(drives)
    if drive_order.index(power_code) < drive_order.index(maneuver_code):
        raise DesignError(
            f"power plant {power_code} is rated below maneuver drive "
            f"{maneuver_code}"
        )
    for kind, code, rule in (
        ("maneuver", maneuver_code, maneuver),
        ("power", power_code, power),
    ):
        volume = _integer(
            rule.get(f"{kind}_millitons"),
            f"small.drive.{code}.{kind}_millitons",
        )
        price = _integer(
            rule.get(f"{kind}_price_credits"),
            f"small.drive.{code}.{kind}_price_credits",
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": f"{kind}-drive" if kind == "maneuver" else "power-plant",
                "id": code,
                "quantity": 1,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    fuel_choice = _table(design.get("fuel"), "design.fuel")
    _check_keys(fuel_choice, {"power_plant_weeks"}, "design.fuel")
    fuel_weeks = _integer(
        fuel_choice.get("power_plant_weeks"),
        "design.fuel.power_plant_weeks",
        1,
    )
    fuel = fuel_weeks * _integer(
        power.get("power_fuel_millitons_per_week"),
        f"small.drive.{power_code}.power_fuel_millitons_per_week",
    )
    totals.add(fuel, 0, discountable=False)
    line_items.append(
        {
            "kind": "fuel",
            "id": "power-plant-fuel",
            "quantity": fuel_weeks,
            "displacement_millitons": fuel,
            "price_credits": 0,
        }
    )

    control_choice = _table(design.get("control"), "design.control")
    _check_keys(
        control_choice,
        {"id", "additional_passengers"},
        "design.control",
    )
    control_id = _text(control_choice.get("id"), "design.control.id")
    try:
        control = controls[control_id]
    except KeyError as error:
        raise DesignError(f"unknown control installation {control_id!r}") from error
    additional_passengers = _integer(
        control_choice.get("additional_passengers", 0),
        "design.control.additional_passengers",
    )
    control_cost_rule = _table(small.get("control_cost"), "small.control_cost")
    control_volume = _integer(
        control.get("displacement_millitons"),
        f"small.control.{control_id}.displacement_millitons",
    ) + additional_passengers * _integer(
        control_cost_rule.get("additional_passenger_millitons"),
        "small.control_cost.additional_passenger_millitons",
    )
    control_increment_price = _integer(
        control_cost_rule.get("price_credits_per_20_hull_tons"),
        "small.control_cost.price_credits_per_20_hull_tons",
    )
    if control_cost_rule.get("rounding") != "up":
        raise DesignError("small-craft control cost must round up")
    base_control_price = ((hull_tons + 19) // 20) * control_increment_price
    control_price = base_control_price + additional_passengers * _integer(
        control_cost_rule.get("additional_passenger_price_credits"),
        "small.control_cost.additional_passenger_price_credits",
    )
    totals.add(control_volume, control_price)
    line_items.append(
        {
            "kind": "control",
            "id": control_id,
            "quantity": 1,
            "displacement_millitons": control_volume,
            "price_credits": control_price,
        }
    )

    computer_choice = _table(design.get("computer"), "design.computer")
    _check_keys(computer_choice, {"id", "options"}, "design.computer")
    computer_id = _text(computer_choice.get("id"), "design.computer.id")
    try:
        computer = computers[computer_id]
    except KeyError as error:
        raise DesignError(f"unknown computer {computer_id!r}") from error
    _require_tl(design_tl, computer, f"computer {computer_id}")
    computer_price = _integer(
        computer.get("price_credits"),
        f"core.computer.{computer_id}.price_credits",
    )
    options = _text_list(
        computer_choice.get("options", []), "design.computer.options"
    )
    for option_id in options:
        try:
            option = computer_options[option_id]
        except KeyError as error:
            raise DesignError(f"unknown computer option {option_id!r}") from error
        computer_price += (
            _integer(
                computer.get("price_credits"),
                f"core.computer.{computer_id}.price_credits",
            )
            * _integer(
                option.get("price_percent_of_computer"),
                f"core.computer_option.{option_id}.price_percent_of_computer",
            )
            // 100
        )
    totals.add(0, computer_price)
    line_items.append(
        {
            "kind": "computer",
            "id": computer_id,
            "quantity": 1,
            "displacement_millitons": 0,
            "price_credits": computer_price,
        }
    )

    for position, choice in enumerate(
        _records(design.get("software"), "design.software"), 1
    ):
        label = f"design.software[{position}]"
        _check_keys(choice, {"id", "level"}, label)
        software_id = _text(choice.get("id"), f"{label}.id")
        level = _integer(choice.get("level"), f"{label}.level", 1)
        try:
            software = software_rules[software_id]
        except KeyError as error:
            raise DesignError(f"unknown software {software_id!r}") from error
        _require_tl(design_tl, software, f"software {software_id}")
        if software_id in {"jump-control", "jump-course-tape"}:
            raise DesignError("small craft cannot install Jump software")
        software_rating = level * _integer(
            software.get("rating_per_level"),
            f"core.software.{software_id}.rating_per_level",
        )
        if software_rating > _integer(
            computer.get("rating"),
            f"core.computer.{computer_id}.rating",
            1,
        ):
            raise DesignError(
                f"software {software_id!r} requires rating {software_rating}"
            )
        price = level * _integer(
            software.get("price_credits_per_level"),
            f"core.software.{software_id}.price_credits_per_level",
        )
        totals.add(0, price)
        line_items.append(
            {
                "kind": "software",
                "id": software_id,
                "quantity": level,
                "displacement_millitons": 0,
                "price_credits": price,
            }
        )
    electronics_id = _text(design.get("electronics"), "design.electronics")
    try:
        electronics = electronics_rules[electronics_id]
    except KeyError as error:
        raise DesignError(f"unknown electronics {electronics_id!r}") from error
    _require_tl(design_tl, electronics, f"electronics {electronics_id}")
    electronics_volume = _integer(
        electronics.get("displacement_millitons"),
        f"core.electronics.{electronics_id}.displacement_millitons",
    )
    electronics_price = _integer(
        electronics.get("price_credits"),
        f"core.electronics.{electronics_id}.price_credits",
    )
    totals.add(electronics_volume, electronics_price)
    line_items.append(
        {
            "kind": "electronics",
            "id": electronics_id,
            "quantity": 1,
            "displacement_millitons": electronics_volume,
            "price_credits": electronics_price,
        }
    )

    installed_equipment: set[str] = set()
    for position, choice in enumerate(
        _records(design.get("equipment"), "design.equipment"), 1
    ):
        label = f"design.equipment[{position}]"
        _check_keys(choice, {"id", "quantity"}, label)
        equipment_id = _text(choice.get("id"), f"{label}.id")
        quantity = _integer(choice.get("quantity"), f"{label}.quantity", 1)
        if equipment_id in installed_equipment:
            raise DesignError(f"duplicate equipment record {equipment_id!r}")
        installed_equipment.add(equipment_id)
        try:
            equipment = equipment_rules[equipment_id]
        except KeyError as error:
            raise DesignError(f"unknown equipment {equipment_id!r}") from error
        volume = quantity * _integer(
            equipment.get("displacement_millitons_per_unit"),
            f"core.equipment.{equipment_id}.displacement_millitons_per_unit",
        )
        price = quantity * _integer(
            equipment.get("price_credits_per_unit"),
            f"core.equipment.{equipment_id}.price_credits_per_unit",
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "equipment",
                "id": equipment_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )
    included_scoops = configuration.get("includes_fuel_scoops")
    if included_scoops is True and "fuel-scoop" in installed_equipment:
        raise DesignError("streamlined hull already includes fuel scoops")
    if configuration.get("may_install_fuel_scoops") is False and (
        "fuel-scoop" in installed_equipment
    ):
        raise DesignError("distributed hull cannot install fuel scoops")

    installed_parameterized: set[str] = set()
    for position, choice in enumerate(
        _records(
            design.get("parameterized_equipment"),
            "design.parameterized_equipment",
        ),
        1,
    ):
        label = f"design.parameterized_equipment[{position}]"
        equipment_id = _text(choice.get("id"), f"{label}.id")
        if equipment_id in installed_parameterized:
            raise DesignError(
                f"parameterized equipment {equipment_id!r} must use one record"
            )
        installed_parameterized.add(equipment_id)
        try:
            equipment = parameterized_equipment_rules[equipment_id]
        except KeyError as error:
            raise DesignError(
                f"unknown parameterized equipment {equipment_id!r}"
            ) from error
        _require_tl(design_tl, equipment, f"equipment {equipment_id}")
        formula = _text(
            equipment.get("formula"),
            f"core.parameterized_equipment.{equipment_id}.formula",
        )
        if formula == "hull-volume-percent":
            _check_keys(choice, {"id", "quantity"}, label)
            units = _integer(
                choice.get("quantity", 1),
                f"{label}.quantity",
                1,
            )
            percent = _integer(
                equipment.get("hull_volume_percent"),
                f"core.parameterized_equipment.{equipment_id}."
                "hull_volume_percent",
                1,
            )
            volume = (hull_millitons * percent + 99) // 100 * units
            price_per_ton = _integer(
                equipment.get("price_credits_per_installed_ton"),
                f"core.parameterized_equipment.{equipment_id}."
                "price_credits_per_installed_ton",
            )
            price = volume * price_per_ton // 1000
        else:
            parameter = _text(
                equipment.get("parameter"),
                f"core.parameterized_equipment.{equipment_id}.parameter",
            )
            _check_keys(choice, {"id", parameter}, label)
            units = _integer(choice.get(parameter), f"{label}.{parameter}", 1)
        if formula == "contained-volume-percent":
            percent = _integer(
                equipment.get("contained_volume_percent"),
                f"core.parameterized_equipment.{equipment_id}."
                "contained_volume_percent",
                1,
            )
            volume = (units * percent + 99) // 100
            price_per_ton = _integer(
                equipment.get("price_credits_per_installed_ton"),
                f"core.parameterized_equipment.{equipment_id}."
                "price_credits_per_installed_ton",
            )
            price = volume * price_per_ton // 1000
        elif formula == "first-unit-plus-additional-units":
            maximum = equipment.get("maximum_units")
            if maximum is not None and units > _integer(
                maximum,
                f"core.parameterized_equipment.{equipment_id}.maximum_units",
                1,
            ):
                raise DesignError(
                    f"parameterized equipment {equipment_id!r} exceeds "
                    f"maximum {maximum}"
                )
            volume = _integer(
                equipment.get("first_unit_millitons"),
                f"core.parameterized_equipment.{equipment_id}."
                "first_unit_millitons",
                1,
            ) + (units - 1) * _integer(
                equipment.get("additional_unit_millitons"),
                f"core.parameterized_equipment.{equipment_id}."
                "additional_unit_millitons",
            )
            price = units * _integer(
                equipment.get("price_credits_per_unit"),
                f"core.parameterized_equipment.{equipment_id}."
                "price_credits_per_unit",
            )
        elif formula != "hull-volume-percent":
            raise DesignError(
                f"small craft cannot use parameterized formula {formula!r}"
            )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "parameterized-equipment",
                "id": equipment_id,
                "quantity": 1,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    airlocks = _integer(design.get("airlocks", 0), "design.airlocks")
    if airlocks:
        airlock = _table(small.get("airlock"), "small.airlock")
        volume = airlocks * _integer(
            airlock.get("displacement_millitons"),
            "small.airlock.displacement_millitons",
        )
        price = airlocks * _integer(
            airlock.get("price_credits"), "small.airlock.price_credits"
        )
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "airlock",
                "id": "small-craft-airlock",
                "quantity": airlocks,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    additional_fire_control = _integer(
        design.get("additional_fire_control_stations", 0),
        "design.additional_fire_control_stations",
    )
    if additional_fire_control > 1:
        raise DesignError("small craft have only one hardpoint")
    if additional_fire_control:
        totals.add(additional_fire_control * 1000, 0)
        line_items.append(
            {
                "kind": "fire-control",
                "id": "additional-station",
                "quantity": additional_fire_control,
                "displacement_millitons": additional_fire_control * 1000,
                "price_credits": 0,
            }
        )

    unused_fire_control = _integer(
        design.get("unused_fire_control_stations", 0),
        "design.unused_fire_control_stations",
    )
    if unused_fire_control > 1:
        raise DesignError("small craft have only one hardpoint")
    if unused_fire_control:
        totals.add(1000, 0)
        line_items.append(
            {
                "kind": "fire-control",
                "id": "unarmed-station",
                "quantity": 1,
                "displacement_millitons": 1000,
                "price_credits": 0,
            }
        )

    mounts = _records(design.get("mounts"), "design.mounts")
    if len(mounts) + unused_fire_control > 1:
        raise DesignError("small craft have only one hardpoint")
    energy_weapon_ids = {"pulse-laser", "beam-laser", "particle-beam"}
    energy_weapons = 0
    for position, choice in enumerate(mounts, 1):
        label = f"design.mounts[{position}]"
        _check_keys(choice, {"id", "weapons", "pop_up", "fixed"}, label)
        mount_id = _text(choice.get("id"), f"{label}.id")
        try:
            mount = mount_rules[mount_id]
        except KeyError as error:
            raise DesignError(f"unknown mount {mount_id!r}") from error
        raw_weapons = choice.get("weapons")
        if not isinstance(raw_weapons, list) or not all(
            isinstance(item, str) and item for item in raw_weapons
        ):
            raise DesignError(f"{label}.weapons must be a list of strings")
        # Repeated installations are valid: a triple turret may contain three
        # instances of the same weapon rule.
        weapons = raw_weapons
        if len(weapons) > _integer(
            mount.get("weapon_capacity"),
            f"core.mount.{mount_id}.weapon_capacity",
            1,
        ):
            raise DesignError("mount weapon capacity exceeded")
        volume = _integer(
            mount.get("displacement_millitons"),
            f"core.mount.{mount_id}.displacement_millitons",
        )
        price = _integer(
            mount.get("price_credits"), f"core.mount.{mount_id}.price_credits"
        )
        fixed = choice.get("fixed", False)
        pop_up = choice.get("pop_up", False)
        if not isinstance(fixed, bool) or not isinstance(pop_up, bool):
            raise DesignError("mount flags must be boolean")
        if fixed and pop_up:
            raise DesignError("mount cannot be fixed and pop-up")
        if fixed:
            volume = 0
            price //= 2
        if pop_up:
            if design_tl < 10:
                raise DesignError("pop-up mount requires TL10")
            volume = 2000
            price += 1000000
        for weapon_id in weapons:
            try:
                weapon = weapon_rules[weapon_id]
            except KeyError as error:
                raise DesignError(f"unknown weapon {weapon_id!r}") from error
            _require_tl(design_tl, weapon, f"weapon {weapon_id}")
            price += _integer(
                weapon.get("price_credits"),
                f"core.weapon.{weapon_id}.price_credits",
            )
            energy_weapons += int(weapon_id in energy_weapon_ids)
        totals.add(volume, price)
        line_items.append(
            {
                "kind": "weapon-mount",
                "id": mount_id,
                "quantity": 1,
                "weapons": weapons,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )
    maximum_energy = None
    for row in _records(
        small.get("energy_weapon_limit"), "small.energy_weapon_limit"
    ):
        first = drive_order.index(
            _text(row.get("first_drive_code"), "energy limit first code")
        )
        last = drive_order.index(
            _text(row.get("last_drive_code"), "energy limit last code")
        )
        if first <= drive_order.index(power_code) <= last:
            maximum_energy = _integer(
                row.get("maximum_weapons"), "energy limit maximum"
            )
            break
    if maximum_energy is None or energy_weapons > maximum_energy:
        raise DesignError(
            f"power plant {power_code} permits {maximum_energy or 0} "
            f"energy weapons; design fits {energy_weapons}"
        )
    fixed_mounts = sum(
        1 for mount in mounts if mount.get("fixed", False) is True
    )
    if additional_fire_control > fixed_mounts:
        raise DesignError(
            "additional small-craft fire-control stations must support "
            "installed fixed mounts"
        )

    for position, choice in enumerate(
        _records(design.get("ammunition"), "design.ammunition"), 1
    ):
        label = f"design.ammunition[{position}]"
        _check_keys(choice, {"id", "quantity"}, label)
        ammunition_id = _text(choice.get("id"), f"{label}.id")
        quantity = _integer(choice.get("quantity"), f"{label}.quantity", 1)
        try:
            ammunition = ammunition_rules[ammunition_id]
        except KeyError as error:
            raise DesignError(f"unknown ammunition {ammunition_id!r}") from error
        units_per_ton = _integer(
            ammunition.get("units_per_ton"),
            f"core.ammunition.{ammunition_id}.units_per_ton",
            1,
        )
        volume = (quantity * 1000 + units_per_ton - 1) // units_per_ton
        price = quantity * _integer(
            ammunition.get("price_credits_per_unit"),
            f"core.ammunition.{ammunition_id}.price_credits_per_unit",
        )
        totals.add(volume, price, discountable=False)
        line_items.append(
            {
                "kind": "ammunition",
                "id": ammunition_id,
                "quantity": quantity,
                "displacement_millitons": volume,
                "price_credits": price,
            }
        )

    crew_records = _records(design.get("crew"), "design.crew")
    crew_total = 0
    for position, record in enumerate(crew_records, 1):
        label = f"design.crew[{position}]"
        _check_keys(record, {"role", "quantity"}, label)
        _text(record.get("role"), f"{label}.role")
        crew_total += _integer(record.get("quantity"), f"{label}.quantity", 1)
    crew_rule = _table(small.get("crew"), "small.crew")
    threshold = _integer(
        crew_rule.get("maximum_hull_tons_for_one_crew"),
        "small.crew.maximum_hull_tons_for_one_crew",
        1,
    )
    minimum_crew = _integer(
        crew_rule.get(
            "minimum_crew_at_or_below"
            if hull_tons <= threshold
            else "minimum_crew_above"
        ),
        "small.crew.minimum",
        1,
    )
    if crew_total < minimum_crew:
        raise DesignError(
            f"small craft requires at least {minimum_crew} crew; design has {crew_total}"
        )
    if crew_total > _integer(
        control.get("crew_capacity"),
        f"small.control.{control_id}.crew_capacity",
        1,
    ):
        raise DesignError("crew exceeds control installation capacity")

    cargo = _integer(design.get("cargo_millitons", 0), "design.cargo_millitons")
    totals.add(cargo, 0)
    if cargo:
        line_items.append(
            {
                "kind": "cargo",
                "id": "cargo-hold",
                "quantity": 1,
                "displacement_millitons": cargo,
                "price_credits": 0,
            }
        )
    if totals.displacement_millitons != hull_millitons:
        difference = totals.displacement_millitons - hull_millitons
        raise DesignError(
            f"small craft displacement differs from hull by {difference} "
            "millitons; no adjustment is permitted"
        )

    discount_percent = (
        _integer(
            _table(core.get("construction"), "core.construction").get(
                "standard_design_discount_percent"
            ),
            "core.construction.standard_design_discount_percent",
        )
        if standard
        else 0
    )
    discounted = (
        totals.discountable_credits * (100 - discount_percent) // 100
    )
    final_price = discounted + totals.undiscounted_credits
    result: dict[str, Any] = {
        "design_id": _text(design.get("design_id"), "design.design_id"),
        "ruleset_id": ruleset_id,
        "hull_millitons": hull_millitons,
        "accounted_displacement_millitons": totals.displacement_millitons,
        "hull_points": hull_tons // 50,
        "structure_points": (hull_tons + 49) // 50,
        "armor_points": armor_points,
        "thrust_g": thrust,
        "fuel_millitons": fuel,
        "hardpoints": 1,
        "hardpoints_used": len(mounts),
        "minimum_crew": minimum_crew,
        "crew": crew_total,
        "construction_weeks": _integer(
            hull.get("construction_weeks"),
            f"small.hull.{hull_id}.construction_weeks",
            1,
        ),
        "pre_discount_price_credits": (
            totals.discountable_credits + totals.undiscounted_credits
        ),
        "discount_credits": totals.discountable_credits - discounted,
        "construction_price_credits": final_price,
        "line_items": line_items,
    }
    assertions = _table(design.get("assertions", {}), "design.assertions")
    _check_keys(
        assertions,
        set(result) - {"design_id", "ruleset_id", "line_items"},
        "design.assertions",
    )
    for field, expected in assertions.items():
        expected = _integer(expected, f"design.assertions.{field}")
        if result[field] != expected:
            raise DesignError(
                f"published assertion {field}={expected} does not match "
                f"rules-derived value {result[field]}"
            )
    return result
