use bevy::{
    color::palettes::css::{DARK_BLUE, LIGHT_GREEN},
    math::{VectorSpace, vec3},
    prelude::*,
    sprite::Sprite,
};

use crate::{particle::Particle, particles_spawning::PARTICLE_RAY};
const SHOW_PARTICLE_VISUALS: bool = true;
const SPEED_VISUALIZATION_SCALE: f32 = 80f32;

pub fn update_particles_visuals(
    mut particles: Query<(&mut Transform, &Particle, &mut Sprite, &mut Text2d)>,
    mut gizmos: Gizmos,
) {
    if !SHOW_PARTICLE_VISUALS {
        return;
    }

    particles
        .iter_mut()
        .for_each(|(mut transform, particle, mut sprite, mut text)| {
            // let t = particle.bonds.len() as f32 / 2f32;

            let t = particle.connected_electrons_needed as f32
                / particle.element().connected_electrons_needed as f32;

            sprite.color = Color::Srgba(Srgba::lerp(DARK_BLUE, LIGHT_GREEN, t));
            text.0 = format!(
                "{}{}",
                particle.element().symbol,
                particle.connected_electrons_needed
            );

            let scale = PARTICLE_RAY
                /* * (pressure_handler::TARGET_DENSITY / particle.density).clamp(0.1f32, 3f32) */;
            transform.scale = vec3(scale, scale, 0f32);

            // for particle_pos in &particle.particles_in_range {
            //     gizmos.line_2d(particle.position_pm, particle_pos.clone(), Srgba::GREEN);
            // }

            for bond in &particle.bonds {
                gizmos.line_2d(particle.position_pm, bond.1.bonded_pos, Srgba::BLACK);
            }
        });
}
