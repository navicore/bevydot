use bevy::prelude::*;
use crate::types::SearchState;

#[derive(Component)]
pub struct CameraController {
    pub speed: f32,
    pub distance: f32,
    pub orbit_angle: f32, // Horizontal rotation around Y axis
    pub pitch_angle: f32, // Vertical rotation
    pub look_at: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 5.0,
            distance: 10.0,
            orbit_angle: 0.0,
            pitch_angle: 0.5, // 0.5 radians (~30 degrees) for nice default view
            look_at: Vec3::ZERO,
        }
    }
}

pub fn setup_camera(commands: &mut Commands, initial_distance: f32, speed: f32) {
    let controller = CameraController {
        distance: initial_distance,
        speed,
        ..Default::default()
    };

    // Calculate initial position based on orbit angles and distance
    let horizontal_distance = controller.distance * controller.pitch_angle.cos();
    let x = horizontal_distance * controller.orbit_angle.cos();
    let y = controller
        .distance
        .mul_add(controller.pitch_angle.sin(), controller.look_at.y);
    let z = horizontal_distance * controller.orbit_angle.sin();

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(x, y, z).looking_at(controller.look_at, Vec3::Y),
        controller,
    ));
}

pub fn camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    search_state: Res<SearchState>,
) {
    // Don't move camera when searching
    if search_state.active {
        return;
    }
    for (mut transform, mut controller) in &mut query {
        let delta = time.delta_secs();
        let mut changed = false;

        // Rotation with Shift + Arrow keys
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                controller.orbit_angle += delta * 2.0;
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                controller.orbit_angle -= delta * 2.0;
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                controller.pitch_angle = (controller.pitch_angle + delta).min(1.4); // Limit to ~80 degrees
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                controller.pitch_angle = (controller.pitch_angle - delta).max(-1.4);
                changed = true;
            }
        } else {
            // Movement with Arrow keys
            let forward = transform.forward();
            let right = transform.right();
            let speed = controller.speed;

            if keyboard_input.pressed(KeyCode::ArrowUp) {
                controller.look_at += forward * speed * delta;
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                controller.look_at -= forward * speed * delta;
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                controller.look_at -= right * speed * delta;
                changed = true;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                controller.look_at += right * speed * delta;
                changed = true;
            }
        }

        // Zoom with +/-
        if keyboard_input.pressed(KeyCode::Equal) || keyboard_input.pressed(KeyCode::NumpadAdd) {
            controller.distance = delta.mul_add(-10.0, controller.distance).max(2.0);
            changed = true;
        }
        if keyboard_input.pressed(KeyCode::Minus) || keyboard_input.pressed(KeyCode::NumpadSubtract)
        {
            controller.distance = delta.mul_add(10.0, controller.distance).min(100.0);
            changed = true;
        }

        // Alternative zoom with PageUp/PageDown
        if keyboard_input.pressed(KeyCode::PageUp) {
            controller.distance = delta.mul_add(-10.0, controller.distance).max(2.0);
            changed = true;
        }
        if keyboard_input.pressed(KeyCode::PageDown) {
            controller.distance = delta.mul_add(10.0, controller.distance).min(100.0);
            changed = true;
        }

        // Update camera position if any changes occurred
        if changed {
            let horizontal_distance = controller.distance * controller.pitch_angle.cos();
            let x = horizontal_distance * controller.orbit_angle.cos();
            let y = controller
                .distance
                .mul_add(controller.pitch_angle.sin(), controller.look_at.y);
            let z = horizontal_distance * controller.orbit_angle.sin();

            transform.translation = Vec3::new(x, y, z) + controller.look_at;
            transform.look_at(controller.look_at, Vec3::Y);
        }
    }
}

pub fn exit_on_q(
    keyboard_input: Res<ButtonInput<KeyCode>>, 
    mut exit: EventWriter<AppExit>,
) {
    // Only Q exits the application
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}
