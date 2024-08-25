use vts_units::{fields::Fields, Scenario, UnitRef, UnitType};

use super::{Lint, LintError, UnitLint};

pub fn mw_has_radar() -> impl Lint {
    let filter = |unit: &UnitRef| {
        unit.unit_type()
            .is_some_and(|t| t == UnitType::MissileWarning)
    };

    let linter = |unit: &UnitRef, scenario: &Scenario| {
        let fields =
            vts_units::fields::access_fields(unit).map_err(|e| LintError::UnitAccessError {
                unit: unit.name().to_string(),
                unit_id: unit.id(),
                reason: e,
            })?;

        let Fields::MissileWarning { radars, .. } = fields else {
            panic!("unit wasn't a missile warning truck!");
        };

        if radars.is_empty() {
            return Ok(vec![(unit, "MW has no Radars!".to_string()).into()]);
        }

        Ok(radars
            .iter()
            .filter_map(|u| {
                let Some(radar) = scenario.units().get(u) else {
                    return Some((unit, format!("MW radar {} does not exist!", u)).into());
                };

                if !radar.unit_type().is_some_and(|t| t == UnitType::SAMRadar) {
                    return Some(
                        (
                            unit,
                            format!(
                                "MW radar {} (id:{}) is not a radar.",
                                radar.name(),
                                radar.id()
                            ),
                        )
                            .into(),
                    );
                }

                None
            })
            .collect())
    };

    UnitLint::new(filter, linter)
}
