use bevy::prelude::*;

pub struct Element {
    pub radious_pm: u16, // pico meters
    pub mass_u: u16,     //  units
    pub name: &'static str,
    pub symbol: &'static str,
    pub color: Srgba,
    pub connected_electrons_needed: u8,

    pub valence_electrons: u8,
}

// for better accuracy other unit than unit for mass might be better
pub const ELEMENTS: [Element; 3] = [
    Element {
        valence_electrons: 6,
        radious_pm: 1, // not right
        mass_u: 16,
        name: "Oxygen",
        symbol: "O",
        connected_electrons_needed: 2,

        color: Srgba::WHITE,
    },
    Element {
        valence_electrons: 1,
        radious_pm: 1, // not right
        mass_u: 1,
        name: "Hydrogen",
        symbol: "H",
        color: Srgba::RED,
        connected_electrons_needed: 1,
    },
    Element {
        valence_electrons: 4,
        radious_pm: 1, // not right
        mass_u: 12,
        name: "Carbon",
        symbol: "C",
        color: Srgba::BLACK,
        connected_electrons_needed: 4,
    },
];
