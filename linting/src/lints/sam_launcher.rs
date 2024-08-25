use vts_units::{
    fields::{self, Fields},
    Scenario, UnitRef, UnitType,
};

use crate::lints::LintError;

use super::{Lint, UnitLint};

pub fn sam_launcher_attached_to_radar() -> impl Lint {
    let filter = |unit: &UnitRef| unit.unit_type().is_some_and(|t| t == UnitType::SAMLauncher);

    let linter = |unit: &UnitRef, scenario: &Scenario| {
        let Fields::SAMLauncher { radars, .. } =
            fields::access_fields(unit).map_err(|e| LintError::UnitAccessError {
                unit: unit.name().to_string(),
                unit_id: unit.id(),
                reason: e,
            })?
        else {
            panic!("unit wasn't a sam launcher");
        };

        if radars.is_empty() {
            return Ok(vec![(unit, "SAM Launcher has no radars!").into()]);
        }

        Ok(radars
            .into_iter()
            .filter_map(|u| {
                let Some(radar) = scenario.units().get(&u) else {
                    return Some(
                        (unit, format!("SAM Launcher's radar id:{u} does not exist!")).into(),
                    );
                };

                if !radar.unit_type().is_some_and(|t| t == UnitType::SAMRadar) {
                    return Some(
                        (
                            unit,
                            format!(
                                "SAM Launcher's radar {} (id:{}) is not a radar!",
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
