use std::collections::HashSet;

use codegen::Faction;
use codegen::Unit;

fn type_override(id: &str) -> Option<String> {
    match id {
        "AMWSTruck Variant" | "EMWSTruck" => Some("MissileWarning".into()),

        "SLAIM120Truck" | "AlliedBackstopSAM" | "PatriotLauncher" | "MAD-4Launcher"
        | "slmrmLauncher" | "samBattery1" => Some("SAMLauncher".into()),

        "WatchmanTruck" | "AlliedEWRadar" | "BSTOPRadar" | "PatRadarTrailer" | "MAD-4Radar"
        | "slmrmRadar" | "SamFCR" | "SamFCR2" => Some("SAMRadar".into()),

        "aIRMDlauncher" | "eIRMDlauncher" => Some("IRMD".into()),

        _ => None,
    }
}

fn name_to_rust_identifier(name: &str) -> String {
    let name = name.replace(" ", "_");
    let name = name.replace("/", "_");
    let name = name.replace("-", "_");

    name
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("expecting one path argument");
    eprintln!("reading from {path}");
    let data = std::fs::read_to_string(path).expect("couldn't read path");
    let root = vts_parsing::parse(&data);

    let units = root.get_node("UNITS").unwrap();

    let (units, ..) = units.nodes().fold(
        (Vec::new(), HashSet::new(), Faction::Allied),
        |(mut units, mut names, faction), u| {
            let mut name = u
                .values
                .get("unitName")
                .unwrap()
                .as_string()
                .unwrap()
                .to_string();

            if names.contains(&name) {
                // this means an allied unit shares the same name.
                // also implies, that we are an enemy unit.
                assert_eq!(faction, Faction::Enemy);

                name = format!("Enemy {name}");
            }

            assert!(
                names.insert(name.clone()),
                "duplicate name, after deduplication"
            );

            let id = u.values.get("unitID").unwrap().as_string().unwrap();
            let rust_name = name_to_rust_identifier(&name);

            units.push(Unit {
                name: name.to_string(),
                id: id.to_string(),
                rust_identifier_name: rust_name,
                faction,
                unit_type: type_override(id),
            });

            match id {
                "MultiplayerSpawn" => (units, names, Faction::Enemy),
                _ => (units, names, faction),
            }
        },
    );

    println!("{}", serde_json::to_string_pretty(&units).unwrap());
}
