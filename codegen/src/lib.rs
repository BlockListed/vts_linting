use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum Faction {
    Allied,
    Enemy,
}

#[derive(Deserialize, Serialize)]
pub struct Unit {
    pub name: String,
    pub id: String,
    pub rust_identifier_name: String,
    pub faction: Faction,
    pub unit_type: Option<String>,
}

fn generate_factions() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Faction {
            Allied,
            Enemy,
        }
    }
}

fn generate_unit_types(units: &[Unit]) -> TokenStream {
    let types_hashset = units.iter().fold(HashSet::new(), |mut set, u| {
        if let Some(ref t) = u.unit_type {
            set.insert(t);
        }
        set
    });

    let types = types_hashset
        .iter()
        .map(|ident| proc_macro2::Ident::new(ident, proc_macro2::Span::call_site()));

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum UnitType {
            #(#types),*
        }
    }
}

fn generate_units(units: &[Unit]) -> TokenStream {
    let identifiers = units
        .iter()
        .map(|u| &u.rust_identifier_name)
        .map(|ident| proc_macro2::Ident::new(ident, proc_macro2::Span::call_site()));
    let ids = units.iter().map(|u| &u.id);

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Unit {
            #( #[doc = #ids]#identifiers ),*
        }
    }
}

fn generate_fromstr_units(units: &[Unit]) -> TokenStream {
    let identifiers = units
        .iter()
        .map(|u| &u.rust_identifier_name)
        .map(|ident| proc_macro2::Ident::new(ident, proc_macro2::Span::call_site()));
    let ids = units
        .iter()
        .map(|u| &u.id)
        .map(|id| proc_macro2::Literal::string(id));

    quote! {
        impl ::std::str::FromStr for Unit {
            type Err = ();

            fn from_str(s: &str) -> Result<Unit, ()> {
                match s {
                    #(#ids => Ok(Unit::#identifiers)),*,
                    _ => Err(()),
                }
            }
        }
    }
}

fn generate_get_unit() -> TokenStream {
    quote! {
        pub fn get_unit(unit: &::vts_parsing::Node) -> Option<Unit> {
            let id = unit.values.get("unitID")?.as_string()?;
            <Unit as ::std::str::FromStr>::from_str(id).ok()
        }
    }
}

fn generate_get_type(units: &[Unit]) -> TokenStream {
    let types_hashmap: HashMap<_, Vec<_>> = units.iter().fold(HashMap::new(), |mut map, u| {
        if let Some(ref t) = u.unit_type {
            map.entry(t).or_default().push(proc_macro2::Ident::new(
                &u.rust_identifier_name,
                proc_macro2::Span::call_site(),
            ));
        }

        map
    });

    let patterns = types_hashmap.iter().map(|(unit_type, idents)| {
        let unit_type = proc_macro2::Ident::new(unit_type, proc_macro2::Span::call_site());
        quote! {
            #(Unit::#idents)|* => Some(UnitType::#unit_type)
        }
    });

    quote! {
        pub fn get_unit_type(unit: &Unit) -> Option<UnitType> {
            match unit {
                #(#patterns),*,
                _ => None,
            }
        }
    }
}

fn generate_get_faction(units: &[Unit]) -> TokenStream {
    let allies = units
        .iter()
        .filter(|u| u.faction == Faction::Allied)
        .map(|u| proc_macro2::Ident::new(&u.rust_identifier_name, proc_macro2::Span::call_site()));
    let enemies = units
        .iter()
        .filter(|u| u.faction == Faction::Enemy)
        .map(|u| proc_macro2::Ident::new(&u.rust_identifier_name, proc_macro2::Span::call_site()));

    quote! {
        pub fn get_unit_faction(unit: &Unit) -> Faction {
            match unit {
                #(Unit::#allies)|* => Faction::Allied,
                #(Unit::#enemies)|* => Faction::Enemy,
            }
        }
    }
}

pub fn generate(from: impl AsRef<Path>) -> TokenStream {
    let data = std::fs::read_to_string(from).expect("couldn't read units json");

    let parsed: Vec<Unit> = serde_json::from_str(&data).expect("invalid json from units json");

    let factions = generate_factions();

    let units = generate_units(&parsed);

    let unit_types = generate_unit_types(&parsed);

    let unit_from_str = generate_fromstr_units(&parsed);

    let get_unit = generate_get_unit();

    let get_unit_type = generate_get_type(&parsed);

    let get_faction = generate_get_faction(&parsed);

    quote! {
        #factions

        #units

        #unit_types

        #unit_from_str

        #get_unit

        #get_unit_type

        #get_faction
    }
}
