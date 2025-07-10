use crate::{
    collisions::resolve_collisions,
    particle::{LookupParticle, Particle},
    particle_grid::{self, get_connected_cells_indexes, pixel_pos_to_gird_pos},
    particles_spawning::{self, PARTICLES_COUNT},
    player_interaction_physics,
};
use bevy::{math::*, prelude::*, window::PrimaryWindow};
use core::f32;

// physics settings
const TIME_SCALE_NS: f32 = 2f32; // nano seconds

const RUN_PHYSICS: bool = true;
const UPDATES_PER_FRAME: u32 = 3;
pub fn handle_particles_physics(
    mut particles: Query<(&mut Transform, &mut Particle)>,
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if !RUN_PHYSICS {
        return;
    }
    let delta_ns = time.delta().as_secs_f32() * TIME_SCALE_NS / UPDATES_PER_FRAME as f32;
    for _ in 0..UPDATES_PER_FRAME {
        let mut particle_positions =
            Vec::with_capacity(particles_spawning::PARTICLES_COUNT as usize);

        for (_, particle) in &particles {
            particle_positions.push(particle.position_pm.to_owned());
        }

        // let connected_cells = particle_grid::calculate_connected_cells_for_every_particle(
        //     &particle_predicted_positions,
        // );
        //
        let grid = particle_grid::split_particles_into_grid(&particle_positions);

        // let densities = &pressure_handler::calculate_density_for_every_particle(
        //     &grid,
        //     &particle_predicted_positions,
        //     &connected_cells,
        // );

        player_interaction_physics::calculate_player_interaction_effect(
            &mut particles,
            &q_window,
            &mouse_buttons,
            &q_camera,
            delta_ns,
        );
        let lookup = create_particle_lookup(&particles);
        handle_chemical_bonds(delta_ns, &mut particles, &grid, lookup);

        particles
            .par_iter_mut()
            .for_each(|(mut transform, mut particle)| {
                if particle.velocity_pm_ns.is_nan() {
                    particle.velocity_pm_ns = particle.last_velocity_pm_ns;
                }
                particle.last_velocity_pm_ns = particle.velocity_pm_ns;

                let s = particle.velocity_pm_ns * delta_ns;

                particle.position_pm += s;
                transform.translation = vec3(particle.position_pm.x, particle.position_pm.y, 0f32);

                resolve_collisions(&mut particle, &mut transform);
            });
    }
}
fn create_particle_lookup(
    particles: &Query<(&mut Transform, &mut Particle)>,
) -> Vec<LookupParticle> {
    let mut output = Vec::with_capacity(PARTICLES_COUNT as usize);
    particles.iter().for_each(|(_, particle)| {
        output.push(LookupParticle {
            element_index: particle.element_index,
            current_unused_valence_electrons: particle.connected_electrons_needed.to_owned(),
            bonds_particle_index: particle.bonds.clone(),
            position_pm: particle.position_pm,
            connected_electrons_needed: particle.connected_electrons_needed,
        });
    });
    output
}
pub const BOND_DISTANCE: f32 = 30f32;

fn handle_chemical_bonds(
    delta_ns: f32,
    particles: &mut Query<(&mut Transform, &mut Particle)>,
    particles_grid: &[Vec<usize>],
    lookup: Vec<LookupParticle>,
) {
    particles.par_iter_mut().for_each(|(_, mut mut_particle)| {
        mut_particle.break_all_out_of_range_bonds();

        mut_particle.particles_in_range.clear();
        for cell in get_connected_cells_indexes(&pixel_pos_to_gird_pos(&mut_particle.position_pm)) {
            for target_index in particles_grid[cell].to_owned() {
                let target_particle = &lookup[target_index];

                if mut_particle.index == target_index {
                    continue;
                }

                let target_bond_option = target_particle
                    .bonds_particle_index
                    .get(&mut_particle.index);
                let mut contains_bond_with_target = false;
                let mut should_break_bond_with_target = false;

                if let Some(mut_bond) = mut_particle.bonds.get_mut(&target_index) {
                    should_break_bond_with_target = mut_bond.should_break;
                    mut_bond.bonded_pos = target_particle.position_pm;
                    contains_bond_with_target = true;

                    keep_bond_distance(delta_ns, &mut mut_particle, target_particle);
                }

                if contains_bond_with_target {
                    if let Some(target_bond) = target_bond_option {
                        // when mut and target does have bond
                        if target_bond.should_break {
                            mut_particle.break_bond_find_bond(&target_index);
                        }
                    } else if should_break_bond_with_target {
                        mut_particle.break_bond_find_bond(&target_index);
                    }
                } else if let Some(target_bond) = target_bond_option {
                    // when mut doesn't but target does have bond
                    if !target_bond.should_break {
                        if target_bond.electrons_used > mut_particle.connected_electrons_needed {
                            // when bond is bigger than you have free electrons
                            // register fake bond that shows that target has to brake the bond that
                            // is too big
                            mut_particle.register_bond(target_index, 0, true, target_particle);
                        } else {
                            mut_particle.register_bond(
                                target_index,
                                target_bond.electrons_used,
                                false,
                                target_particle,
                            );
                        }
                    }
                } else {
                    // when mut and target doesn't have bond
                    try_creating_bond(&mut mut_particle, target_particle, target_index);
                }

                mut_particle
                    .particles_in_range
                    .push(target_particle.position_pm);
            }
        }
    });
}
fn try_creating_bond(
    mut_particle: &mut Particle,
    target_particle: &LookupParticle,
    target_index: usize,
) {
    if mut_particle
        .position_pm
        // use distance squared for speed
        .distance(target_particle.position_pm)
        > BOND_DISTANCE
    {
        return;
    }
    // this is very naive and needs to be changed

    let connected_electrons = mut_particle
        .connected_electrons_needed
        .min(target_particle.connected_electrons_needed);
    if connected_electrons == 0 {
        return;
    }

    mut_particle.register_bond(target_index, connected_electrons, false, target_particle);
}
const BOND_DISTANCE_KEEPING_STRENGTH: f32 = 10f32;
fn keep_bond_distance(
    delta_ns: f32,
    mut_particle: &mut Particle,
    target_particle: &LookupParticle,
) {
    if target_particle.position_pm.x == f32::NAN || mut_particle.position_pm.x == f32::NAN {
        return;
    }

    let needed_distance_change = mut_particle
        .position_pm
        .distance(target_particle.position_pm)
        - BOND_DISTANCE;

    let force_strength = BOND_DISTANCE_KEEPING_STRENGTH * needed_distance_change;
    let force_direction = (target_particle.position_pm - mut_particle.position_pm).normalize();

    mut_particle.position_pm += force_strength * force_direction  * delta_ns/* / mut_particle.element().mass_u as f32 */;
    // println!(
    //     "bond str: {}, {}, {} - {}",
    //     force_strength, force_direction, target_particle.position_pm, mut_particle.position_pm
    // );
}
