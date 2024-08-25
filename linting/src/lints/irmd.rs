use vts_units::{
    fields::{self, Fields},
    Scenario, UnitRef, UnitType,
};

use super::{Lint, LintError, UnitLint};

pub fn irmd_linked_to_by_mw() -> impl Lint {
    let filter = |unit: &UnitRef| unit.unit_type().is_some_and(|t| t == UnitType::IRMD);

    let linter = |unit: &UnitRef, scenario: &Scenario| {
        let our_id = unit.id();

        let has_mw = scenario.units().values().try_fold(false, |acc, u| {
            if !u.unit_type().is_some_and(|t| t == UnitType::MissileWarning) {
                return Ok(acc);
            }

            let Fields::MissileWarning {
                missile_defenses, ..
            } = fields::access_fields(u).map_err(|e| LintError::UnitAccessError {
                unit: unit.name().to_string(),
                unit_id: unit.id(),
                reason: e,
            })?
            else {
                panic!("this should be guaranteed to be an MW");
            };

            if missile_defenses.contains(&our_id) {
                return Ok(true);
            }

            Ok(acc)
        })?;

        if !has_mw {
            return Ok(vec![
                (unit, "IRMD has no MW linking to it!".to_string()).into()
            ]);
        }

        Ok(vec![])
    };

    UnitLint::new(filter, linter)
}
