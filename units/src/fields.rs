use vts_parsing::Value;

use crate::UnitRef;

pub enum Fields {
    SAMRadar {
        engage_enemies: bool,
    },
    SAMLauncher {
        engage_enemies: bool,
        radars: Vec<i64>,
    },
    MissileWarning {
        engage_enemies: bool,
        radars: Vec<i64>,
        decoys: Vec<i64>,
        missile_defenses: Vec<i64>,
        jammers: Vec<i64>,
        units_to_defend: Vec<i64>,
    },
    IRMD {
        engage_enemies: bool,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum AccessFieldsError {
    #[error("Missing field {field}.")]
    MissingFieldError { field: &'static str },
    #[error("Field {field} had invalid type.")]
    FieldInvalidType {
        field: &'static str,
        expected: &'static str,
        found: &'static str,
    },
    #[error("Couldn't parse {field}, because {reason}.")]
    ParseFieldError {
        field: &'static str,
        reason: &'static str,
    },
    #[error("We haven't defined fields for this unit type")]
    InvalidUnitType,
    #[error("Unit has no type")]
    UnitMissingType,
}

fn get_unit_field<'a>(
    unit: &'a UnitRef<'a>,
    field: &'static str,
) -> Result<&'a Value, AccessFieldsError> {
    unit.get_field(field)
        .ok_or(AccessFieldsError::MissingFieldError { field })
}

fn get_engage_enemies(unit: &UnitRef) -> Result<bool, AccessFieldsError> {
    let engage_enemies = get_unit_field(unit, "engageEnemies")?;
    let engage_enemies = engage_enemies
        .as_bool()
        .ok_or(AccessFieldsError::FieldInvalidType {
            field: "engageEnemies",
            expected: "bool",
            found: engage_enemies.get_type(),
        })?;

    Ok(engage_enemies)
}

fn get_instance_id_list_field(
    unit: &UnitRef,
    field: &'static str,
) -> Result<Vec<i64>, AccessFieldsError> {
    let raw_list = get_unit_field(unit, field)?;

    if let Value::Null = raw_list {
        return Ok(vec![]);
    }

    let raw_list = raw_list
        .as_string()
        .ok_or(AccessFieldsError::FieldInvalidType {
            field,
            expected: "string",
            found: raw_list.get_type(),
        })?;

    raw_list
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<i64>())
        .map(|e| {
            e.map_err(|_| AccessFieldsError::ParseFieldError {
                field,
                reason: "invalid number in instance id list",
            })
        })
        .map(|e| {
            e.and_then(|id| {
                if id < 0 {
                    Err(AccessFieldsError::ParseFieldError {
                        field,
                        reason: "negative number in instance id list",
                    })
                } else {
                    Ok(id)
                }
            })
        })
        .collect()
}

fn parse_sam_radar(unit: &UnitRef) -> Result<Fields, AccessFieldsError> {
    assert_eq!(unit.unit_type(), Some(crate::UnitType::SAMRadar));

    let engage_enemies = get_engage_enemies(unit)?;

    Ok(Fields::SAMRadar { engage_enemies })
}

fn parse_sam_launcher(unit: &UnitRef) -> Result<Fields, AccessFieldsError> {
    assert_eq!(unit.unit_type(), Some(crate::UnitType::SAMLauncher));

    let engage_enemies = get_engage_enemies(unit)?;

    let radars = get_instance_id_list_field(unit, "radarUnits")?;

    Ok(Fields::SAMLauncher { engage_enemies, radars })
}

fn parse_missile_warning(unit: &UnitRef) -> Result<Fields, AccessFieldsError> {
    assert_eq!(unit.unit_type(), Some(crate::UnitType::MissileWarning));

    let engage_enemies = get_engage_enemies(unit)?;

    let radars = get_instance_id_list_field(unit, "radarUnits")?;
    let decoys = get_instance_id_list_field(unit, "decoyUnits")?;
    let units_to_defend = get_instance_id_list_field(unit, "unitsToDefend")?;
    let missile_defenses = get_instance_id_list_field(unit, "defenseUnits")?;
    let jammers = get_instance_id_list_field(unit, "jammerUnits")?;

    Ok(Fields::MissileWarning { engage_enemies, radars, decoys, missile_defenses, jammers, units_to_defend })
}

fn parse_irmd(unit: &UnitRef) -> Result<Fields, AccessFieldsError> {
    assert_eq!(unit.unit_type(), Some(crate::UnitType::IRMD));

    let engage_enemies = get_engage_enemies(unit)?;

    Ok(Fields::IRMD { engage_enemies })
}

pub fn access_fields(unit: &UnitRef) -> Result<Fields, AccessFieldsError> {
    let Some(t) = unit.unit_type() else {
        return Err(AccessFieldsError::UnitMissingType);
    };

    #[allow(unreachable_patterns)]
    match t {
        crate::UnitType::SAMRadar => parse_sam_radar(unit),
        crate::UnitType::SAMLauncher => parse_sam_launcher(unit),
        crate::UnitType::MissileWarning => parse_missile_warning(unit),
        crate::UnitType::IRMD => parse_irmd(unit),
        _ => Err(AccessFieldsError::InvalidUnitType),
    }
}
