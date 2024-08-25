use lints::{Lint, Lints};
use vts_parsing::parse::try_parse;
use vts_units::Scenario;

pub mod lints;

fn main() {
    let file = std::env::args_os().nth(1).expect("1 argument expected");

    let data = std::fs::read_to_string(file).unwrap();

    let parsed = try_parse(&data).unwrap();

    let scenario: Scenario = (&parsed).try_into().unwrap();

    let mut lints = Lints::default();
    lints
        .add_lint(lints::mw::mw_has_radar())
        .add_lint(lints::irmd::irmd_linked_to_by_mw())
        .add_lint(lints::sam_launcher::sam_launcher_attached_to_radar());

    for w in lints.lint(&scenario).unwrap() {
        println!("Lint Warning: {:?}", w);
    }
}
