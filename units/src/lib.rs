use std::collections::HashMap;

use vts_parsing::{Node, Value};

include!(concat!(env!("OUT_DIR"), "/units.rs"));

pub mod fields;

pub struct UnitRef<'a> {
    id: i64,
    unit: Unit,
    unit_faction: Faction,
    unit_type: Option<UnitType>,

    name: String,
    fields: &'a Node,
}

#[derive(thiserror::Error, Debug)]
pub enum ToUnitRefError {
    #[error("Unit was not valid unit")]
    NotValidUnit,
    #[error("Unit id missing or invalid")]
    InvalidUnitID,
    #[error("Unit name missing or invalid")]
    InvalidUnitName,
    #[error("Unit fields missing")]
    MissingUnitFields,
}

impl<'a> TryFrom<&'a Node> for UnitRef<'a> {
    type Error = ToUnitRefError;

    fn try_from(node: &'a Node) -> Result<UnitRef<'a>, ToUnitRefError> {
        let unit = get_unit(node).ok_or(ToUnitRefError::NotValidUnit)?;
        let unit_faction = get_unit_faction(&unit);
        let unit_type = get_unit_type(&unit);

        let id = node
            .values
            .get("unitInstanceID")
            .ok_or(ToUnitRefError::InvalidUnitID)?
            .as_number()
            .ok_or(ToUnitRefError::InvalidUnitID)
            .and_then(|n| {
                if n < 0 {
                    Err(ToUnitRefError::InvalidUnitID)
                } else {
                    Ok(n)
                }
            })?;

        let name = node
            .values
            .get("unitName")
            .ok_or(ToUnitRefError::InvalidUnitName)?
            .as_string()
            .ok_or(ToUnitRefError::InvalidUnitName)?
            .to_string();
        let fields = node
            .get_node("UnitFields")
            .ok_or(ToUnitRefError::MissingUnitFields)?;

        Ok(UnitRef {
            id,
            unit,
            unit_faction,
            unit_type,

            name,
            fields,
        })
    }
}

impl<'a> UnitRef<'a> {
    pub fn unit(&self) -> Unit {
        self.unit
    }

    pub fn faction(&self) -> Faction {
        self.unit_faction
    }

    pub fn unit_type(&self) -> Option<UnitType> {
        self.unit_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_field(&self, k: &str) -> Option<&Value> {
        self.fields.values.get(k)
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

pub struct Scenario<'a> {
    units: HashMap<i64, UnitRef<'a>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ToScenarioError {
    #[error("Not CustomScenario")]
    NotCustomScenario,
    #[error("UNITS node missing from route")]
    UnitsMissing,
    #[error("Unit could not be converted to UnitRef")]
    InvalidUnit {
        unit: Node,
        #[source]
        reason: ToUnitRefError,
    },
}

impl<'a> TryFrom<&'a Node> for Scenario<'a> {
    type Error = ToScenarioError;

    fn try_from(node: &'a Node) -> Result<Scenario<'a>, ToScenarioError> {
        if node.name != "CustomScenario" {
            return Err(ToScenarioError::NotCustomScenario);
        }

        let units = node
            .get_node("UNITS")
            .ok_or(ToScenarioError::UnitsMissing)?;

        let units = units
            .nodes()
            .map(|n| {
                let unit_ref = UnitRef::try_from(n).map_err(|e| (n.clone(), e))?;
                let key = unit_ref.id();

                Ok((key, unit_ref))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|(n, e)| ToScenarioError::InvalidUnit { unit: n, reason: e })?;

        Ok(Scenario { units })
    }
}

impl<'a> Scenario<'a> {
    pub fn units(&self) -> &HashMap<i64, UnitRef> {
        &self.units
    }
}
