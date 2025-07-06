use std::usize;

use bevy::{math::bool, prelude::*, utils::HashMap};

use crate::element::{ELEMENTS, Element};
#[derive(Clone)]
pub enum BondType {
    Covalent,
    Ionic, // for later
}
#[derive(Clone)]
pub struct Bond {
    pub bond_type: BondType,
    pub electrons_used: u8,
    pub should_break: bool,
}
#[derive(Component)]
pub(crate) struct Particle {
    pub element_index: u8,
    pub velocity_pm_ns: Vec2,      // pm / ns -> pico meters / nano seconds
    pub last_velocity_pm_ns: Vec2, // pm / ns -> pico meters / nano seconds
    pub index: usize,
    pub bonds: HashMap<usize, Bond>,
    pub predicted_position_pm: Vec2, //pico meters

    pub connected_electrons_needed: u8,
    pub current_unused_valence_electrons: u8,
}
#[derive(Clone)]
pub struct LookupParticle {
    pub element_index: u8,
    pub current_unused_valence_electrons: u8,
    pub bonds_particle_index: HashMap<usize, Bond>,
    pub connected_electrons_needed: u8,

    pub predicted_position_pm: Vec2, //pico meters
}
impl Particle {
    pub fn new(velocity_ms: Vec2, index: usize, element_index: u8, element: &Element) -> Particle {
        Particle {
            bonds: HashMap::new(),
            element_index,
            velocity_pm_ns: velocity_ms,
            last_velocity_pm_ns: Vec2::ZERO,
            index,
            predicted_position_pm: Vec2::ZERO,
            current_unused_valence_electrons: element.valence_electrons,
            connected_electrons_needed: element.connected_electrons_needed,
        }
    }
    pub fn element(&self) -> &Element {
        &ELEMENTS[self.element_index as usize]
    }
    pub fn break_bond(&mut self, bonds_map_key: &usize, bond: &Bond) {
        self.bonds.remove(bonds_map_key);
        self.connected_electrons_needed += bond.electrons_used;
        self.current_unused_valence_electrons += bond.electrons_used;
    }
    pub fn register_bond(
        &mut self,
        paritcle_index: usize,
        electrons_connected: u8,
        should_break: bool,
    ) {
        self.current_unused_valence_electrons -= electrons_connected;
        self.connected_electrons_needed -= electrons_connected;
        self.bonds.insert(
            paritcle_index,
            Bond {
                bond_type: BondType::Covalent,
                electrons_used: electrons_connected,
                should_break,
            },
        );
    }
}
