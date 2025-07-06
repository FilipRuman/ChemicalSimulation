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
        let mut particle_predicted_positions =
            Vec::with_capacity(particles_spawning::PARTICLES_COUNT as usize);

        for (_, particle) in &particles {
            particle_predicted_positions.push(particle.predicted_position_pm.to_owned());
        }

        // let connected_cells = particle_grid::calculate_connected_cells_for_every_particle(
        //     &particle_predicted_positions,
        // );
        //
        let grid = particle_grid::split_particles_into_grid(&particle_predicted_positions);

        // let densities = &pressure_handler::calculate_density_for_every_particle(
        //     &grid,
        //     &particle_predicted_positions,
        //     &connected_cells,
        // );

        calculate_player_interaction_effect(
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

                transform.translation += vec3(s.x, s.y, 0f32);
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
            current_unused_valence_electrons: particle.current_unused_valence_electrons.to_owned(),
            bonds_particle_index: particle.bonds.clone(),
            predicted_position_pm: particle.predicted_position_pm,
            connected_electrons_needed: particle.connected_electrons_needed,
        });
    });
    output
}
const BOND_DISTANCE: f32 = 2f32;

fn handle_chemical_bonds(
    delta_ns: f32,
    particles: &mut Query<(&mut Transform, &mut Particle)>,
    particles_grid: &[Vec<usize>],
    lookup: Vec<LookupParticle>,
) {
    particles.par_iter_mut().for_each(|(_, mut mut_particle)| {
        for cell in
            get_connected_cells_indexes(&pixel_pos_to_gird_pos(&mut_particle.predicted_position_pm))
        {
            for target_index in particles_grid[cell].to_owned() {
                let target_particle = &lookup[target_index];

                //INFO: check if it is connected to you and if you are connected to it
                match target_particle
                    .bonds_particle_index
                    .get(&mut_particle.index)
                {
                    Some(bond) => {
                        if bond.should_break {
                            mut_particle.break_bond(&target_index, bond);
                        }
                        if !mut_particle.bonds.contains_key(&target_index) {
                            if bond.electrons_used > mut_particle.connected_electrons_needed {
                                mut_particle.register_bond(target_index, 0, true);
                            } else {
                                mut_particle.register_bond(
                                    target_index,
                                    bond.electrons_used,
                                    false,
                                );
                            }
                        }
                        keep_bond_distance(delta_ns, &mut mut_particle, target_particle);
                        continue;
                    }
                    _ => {
                        let mut bond_to_break = None;
                        if let Some(mut_bond) = mut_particle.bonds.get(&target_index) {
                            if mut_bond.to_owned().should_break {
                                bond_to_break = Some(mut_bond);
                            }
                        }
                        if let Some(bond) = bond_to_break {
                            mut_particle.break_bond(&target_index,& bond);
                        }
                    }
                }

                try_creating_bond(&mut mut_particle, target_particle, target_index);
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
        .predicted_position_pm
        // use distance squared for speed
        .distance(target_particle.predicted_position_pm)
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

    mut_particle.register_bond(target_index, connected_electrons, false);
}
const BOND_DISTANCE_KEEPING_STRENGTH: f32 = 0.01f32;
fn keep_bond_distance(
    delta_ns: f32,
    mut_particle: &mut Particle,
    target_particle: &LookupParticle,
) {
    let needed_distance = mut_particle
        .predicted_position_pm
        .distance(target_particle.predicted_position_pm)
        - BOND_DISTANCE;

    let force_lenght = BOND_DISTANCE_KEEPING_STRENGTH * needed_distance.squared();
    let force_direction =
        (target_particle.predicted_position_pm - mut_particle.predicted_position_pm).normalize();
    mut_particle.velocity_pm_ns +=
        force_lenght * force_direction * delta_ns / mut_particle.element().mass_u as f32;
}

fn calculate_player_interaction_effect(
    particles: &mut Query<(&mut Transform, &mut Particle)>,
    q_window: &Query<'_, '_, &Window, With<PrimaryWindow>>,
    mouse_buttons: &Res<'_, ButtonInput<MouseButton>>,
    q_camera: &Query<'_, '_, (&Camera, &GlobalTransform)>,
    delta: f32,
) {
    // interactions
    let mut use_interaction: bool = true;
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    let cursor_option = window.cursor_position();
    let mouse_position = match cursor_option {
        Some(pos) => camera
            .viewport_to_world(camera_transform, pos)
            .map(|ray| ray.origin.truncate())
            .unwrap(),
        None => {
            use_interaction = false;
            Vec2::ZERO
        }
    };

    let force_sign;
    if mouse_buttons.pressed(MouseButton::Right) {
        force_sign = -1f32;
    } else if mouse_buttons.pressed(MouseButton::Left) {
        // disabled because not working good enough
        use_interaction = false;
        force_sign = 0.5f32;
    } else {
        use_interaction = false;
        force_sign = 0f32;
    };

    particles.par_iter_mut().for_each(|(_, mut particle)| {
        let interaction_force = match use_interaction {
            true => player_interaction_physics::calculate_interaction_force(
                particle.predicted_position_pm,
                mouse_position,
                force_sign,
                particle.velocity_pm_ns,
            ),
            false => Vec2::ZERO,
        };

        let force = interaction_force;
        let acceleration = force / particle.element().mass_u as f32;
        particle.velocity_pm_ns += acceleration * delta;
    });
}
