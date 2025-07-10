use bevy::{prelude::*, window::PrimaryWindow};

use crate::particle::Particle;

const INTERACTION_STRENGTH: f32 = 900000f32;
const MAX_INTERACTION_DIST_SQRT: f32 = 90000f32;
pub fn calculate_interaction_force(
    pos: Vec2,
    mouse_pos: Vec2,
    force_sign: f32,
    velocity: Vec2,
) -> Vec2 {
    let dist = mouse_pos.distance_squared(pos);
    if dist > MAX_INTERACTION_DIST_SQRT || dist == 0f32 {
        return Vec2::ONE;
    }

    let dir = (mouse_pos - pos) / dist;

    let strength = (INTERACTION_STRENGTH * force_sign - velocity) / dist.max(80f32);

    strength * dir
}

pub fn calculate_player_interaction_effect(
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
            true => calculate_interaction_force(
                particle.position_pm,
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
