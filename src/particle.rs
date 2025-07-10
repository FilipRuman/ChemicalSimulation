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
    pub bonded_pos: Vec2, // only for connection Gizmos
}
#[derive(Component)]
pub(crate) struct Particle {
    pub element_index: u8,
    pub velocity_pm_ns: Vec2,      // pm / ns -> pico meters / nano seconds
    pub last_velocity_pm_ns: Vec2, // pm / ns -> pico meters / nano seconds
    pub index: usize,
    pub bonds: HashMap<usize, Bond>,
    pub position_pm: Vec2, //pico meters
    // for debuging
    pub particles_in_range: Vec<Vec2>,

    pub connected_electrons_needed: u8,
}
#[derive(Clone)]
pub struct LookupParticle {
    pub element_index: u8,
    pub current_unused_valence_electrons: u8,
    pub bonds_particle_index: HashMap<usize, Bond>,
    pub connected_electrons_needed: u8,

    pub position_pm: Vec2, //pico meters
}
impl Particle {
    pub fn new(
        velocity_ms: Vec2,
        index: usize,
        element_index: u8,
        element: &Element,
        position_pm: Vec2,
    ) -> Particle {
        Particle {
            bonds: HashMap::new(),
            element_index,
            velocity_pm_ns: velocity_ms,
            last_velocity_pm_ns: Vec2::ZERO,
            index,
            position_pm,
            connected_electrons_needed: element.connected_electrons_needed,
            particles_in_range: Vec::new(),
        }
    }
    pub fn element(&self) -> &Element {
        &ELEMENTS[self.element_index as usize]
    }
    pub fn break_bond(&mut self, bonds_map_key: &usize, bond: &Bond) {
        self.bonds.remove(bonds_map_key);
        self.connected_electrons_needed += bond.electrons_used;
    }

    // this is the same thing but just slower
    pub fn break_bond_find_bond(&mut self, bonds_map_key: &usize) {
        let bond = &self.bonds[bonds_map_key];
        self.connected_electrons_needed += bond.electrons_used;

        self.bonds.remove(bonds_map_key);
    }
    pub fn register_bond(
        &mut self,
        paritcle_index: usize,
        electrons_connected: u8,
        should_break: bool,
        target_particle: &LookupParticle,
    ) {
        self.connected_electrons_needed -= electrons_connected;
        self.bonds.insert(
            paritcle_index,
            Bond {
                bond_type: BondType::Covalent,
                electrons_used: electrons_connected,
                should_break,
                bonded_pos: target_particle.position_pm,
            },
        );
    }
    pub fn break_all_out_of_range_bonds(&mut self) {
        for bond in self.bonds.to_owned() {
            if bond.1.bonded_pos.distance(self.position_pm) > crate::particle_physics::BOND_DISTANCE
            {
                self.break_bond(&bond.0, &bond.1);
            }
        }
    }
}
