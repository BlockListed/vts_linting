use std::any::Any;

use vts_units::fields::AccessFieldsError;
use vts_units::{Scenario, UnitRef};

pub mod irmd;
pub mod mw;
pub mod sam_launcher;

#[derive(Debug)]
pub struct Warning {
    pub unit_name: String,
    pub unit_id: i64,
    pub description: String,
}

impl<'a, S: Into<String>> From<(&'a UnitRef<'a>, S)> for Warning {
    fn from((unit, description): (&'a UnitRef<'a>, S)) -> Self {
        Warning {
            unit_name: unit.name().to_string(),
            unit_id: unit.id(),
            description: description.into(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LintError {
    #[error("Couldn't access fields of {unit} (id:{unit_id})")]
    UnitAccessError {
        unit: String,
        unit_id: i64,
        reason: AccessFieldsError,
    },
}

pub trait Lint {
    fn lint(&self, scenario: &Scenario) -> Result<Vec<Warning>, LintError>;
}

pub trait AnyLint: Lint + Any {}

impl<T: Lint + 'static> AnyLint for T {}

#[derive(Default)]
pub struct Lints {
    lints: Vec<Box<dyn AnyLint>>,
}

impl Lints {
    pub fn add_lint<T: AnyLint>(&mut self, lint: T) -> &mut Self {
        self.lints.push(Box::new(lint));
        self
    }
}

impl Lint for Lints {
    fn lint(&self, scenario: &Scenario) -> Result<Vec<Warning>, LintError> {
        self.lints
            .iter()
            .map(|l| l.lint(scenario))
            .try_fold(Vec::new(), |mut acc, res| {
                res.map(|vec| {
                    acc.extend(vec);
                    acc
                })
            })
    }
}

pub struct UnitLint<
    F: Fn(&UnitRef) -> bool,
    G: Fn(&UnitRef, &Scenario) -> Result<Vec<Warning>, LintError>,
> {
    filter: F,
    linter: G,
}

impl<F: Fn(&UnitRef) -> bool, G: Fn(&UnitRef, &Scenario) -> Result<Vec<Warning>, LintError>>
    UnitLint<F, G>
{
    pub fn new(filter: F, linter: G) -> Self {
        Self { filter, linter }
    }
}

impl<F: Fn(&UnitRef) -> bool, G: Fn(&UnitRef, &Scenario) -> Result<Vec<Warning>, LintError>> Lint
    for UnitLint<F, G>
{
    fn lint(&self, scenario: &Scenario) -> Result<Vec<Warning>, LintError> {
        scenario
            .units()
            .values()
            .filter(|u| (self.filter)(u))
            .map(|u| (self.linter)(u, scenario))
            .try_fold(Vec::new(), |mut acc, res| {
                res.map(|vec| {
                    acc.extend(vec);
                    acc
                })
            })
    }
}
