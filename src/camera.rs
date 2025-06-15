use crate::types::SearchState;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Update, keyboard_camera_controls)
            .add_systems(Update, exit_on_q)
            .add_systems(Update, debug_camera_state);
    }
}

pub fn setup_camera(commands: &mut Commands, initial_distance: f32, _speed: f32) {
    // Spawn camera with PanOrbitCamera component
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, initial_distance * 0.5, initial_distance))
            .looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            // Core configuration
            focus: Vec3::ZERO,
            radius: Some(initial_distance),
            yaw: Some(0.0),
            pitch: Some(0.5),
            
            // Initialize targets to match
            target_focus: Vec3::ZERO,
            target_radius: initial_distance,
            target_yaw: 0.0,
            target_pitch: 0.5,
            
            // Mouse button configuration
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            
            // Sensitivity settings
            pan_sensitivity: 1.0,
            orbit_sensitivity: 1.0,
            zoom_sensitivity: 0.5,
            
            // Smoothing
            pan_smoothness: 0.8,
            orbit_smoothness: 0.8,
            zoom_smoothness: 0.8,
            
            // Limits
            pitch_upper_limit: Some(1.4),
            pitch_lower_limit: Some(-1.4),
            
            // Make sure it's enabled
            enabled: true,
            
            ..default()
        },
    ));
}

fn debug_camera_state(
    cameras: Query<&PanOrbitCamera>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        for cam in &cameras {
            eprintln!("Camera state:");
            eprintln!("  enabled: {}", cam.enabled);
            eprintln!("  focus: {:?}", cam.focus);
            eprintln!("  yaw: {:?}", cam.yaw);
            eprintln!("  pitch: {:?}", cam.pitch);
            eprintln!("  radius: {:?}", cam.radius);
        }
    }
}

pub fn keyboard_camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cameras: Query<&mut PanOrbitCamera>,
    search_state: Res<SearchState>,
) {
    for mut cam in &mut cameras {
        // Disable camera when searching
        cam.enabled = !search_state.active;
        
        if search_state.active {
            continue;
        }
        
        let delta = time.delta_secs();
        let pan_speed = 5.0 * delta;
        let rotation_speed = 2.0 * delta;
        let zoom_speed = 10.0 * delta;
        
        // Get current yaw for directional movement
        let current_yaw = cam.yaw.unwrap_or(cam.target_yaw);
        
        // More intuitive controls:
        // Arrow keys without shift = pan camera view
        // Arrow keys with shift = orbit around focus point
        
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
            // Orbit mode: Rotate camera around the focus point
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                cam.target_yaw -= rotation_speed;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                cam.target_yaw += rotation_speed;
            }
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                cam.target_pitch = (cam.target_pitch + rotation_speed).min(1.4);
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                cam.target_pitch = (cam.target_pitch - rotation_speed).max(-1.4);
            }
        } else {
            // Pan mode: Move the camera and focus together
            // Calculate movement in world space based on camera orientation
            let forward = Vec3::new(current_yaw.sin(), 0.0, -current_yaw.cos());
            let right = Vec3::new(current_yaw.cos(), 0.0, current_yaw.sin());
            
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                // Move forward (into the scene)
                cam.target_focus += forward * pan_speed;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                // Move backward
                cam.target_focus -= forward * pan_speed;
            }
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                // Move left
                cam.target_focus -= right * pan_speed;
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                // Move right
                cam.target_focus += right * pan_speed;
            }
        }
        
        // Zoom with +/- and PageUp/PageDown
        if keyboard_input.pressed(KeyCode::Equal) || keyboard_input.pressed(KeyCode::NumpadAdd) 
            || keyboard_input.pressed(KeyCode::PageUp) {
            cam.target_radius = (cam.target_radius - zoom_speed).max(2.0);
        }
        if keyboard_input.pressed(KeyCode::Minus) || keyboard_input.pressed(KeyCode::NumpadSubtract)
            || keyboard_input.pressed(KeyCode::PageDown) {
            cam.target_radius = (cam.target_radius + zoom_speed).min(100.0);
        }
    }
}

pub fn exit_on_q(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    // Only Q exits the application
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}