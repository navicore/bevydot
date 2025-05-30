#![allow(clippy::cast_precision_loss)] // We accept precision loss for f32 conversions
#![allow(clippy::needless_pass_by_value)] // Bevy systems require owned Res parameters
#![allow(clippy::multiple_crate_versions)] // Bevy dependencies have multiple versions

use bevy::prelude::*;
use clap::Parser;
use std::io::{self, IsTerminal, Read};

mod camera;
mod parser;
mod types;
mod ui;
mod visualization;

use camera::{camera_controls, exit_on_esc_or_q, setup_camera};
use parser::parse_dot_file;
use types::{CameraSettings, DotContent};
use ui::{create_node_labels, setup_ui, update_node_label_positions};
use visualization::{create_graph_visualization, update_edge_positions};

#[derive(Parser, Debug)]
#[command(author, version, about = "Explore your Graphviz dot files in interactive 3D space", long_about = None)]
struct Args {
    /// Optional dot file path. If not provided, reads from stdin.
    file: Option<String>,

    /// Initial camera distance from center
    #[arg(short, long, default_value = "25.0")]
    distance: f32,

    /// Camera movement speed
    #[arg(short, long, default_value = "5.0")]
    speed: f32,
}

fn main() {
    let args = Args::parse();

    // Read dot content from file or stdin
    let dot_content = args.file.map_or_else(
        || {
            if io::stdin().is_terminal() {
                eprintln!("Error: No input provided. Either specify a file or pipe data to stdin.");
                eprintln!("Usage: dotspace [FILE] or command | dotspace");
                std::process::exit(1);
            } else {
                // Read from stdin if it's piped
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
                    eprintln!("Error reading from stdin: {e}");
                    std::process::exit(1);
                });
                buffer
            }
        },
        |filename| {
            std::fs::read_to_string(&filename).unwrap_or_else(|e| {
                eprintln!("Error reading file '{filename}': {e}");
                std::process::exit(1);
            })
        },
    );

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DotContent(dot_content))
        .insert_resource(CameraSettings {
            distance: args.distance,
            speed: args.speed,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controls)
        .add_systems(Update, exit_on_esc_or_q)
        .add_systems(Update, update_edge_positions)
        .add_systems(Update, create_node_labels)
        .add_systems(Update, update_node_label_positions)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dot_content: Res<DotContent>,
    camera_settings: Res<CameraSettings>,
) {
    // Parse the dot content
    let graph_data = parse_dot_file(&dot_content.0);

    // Setup camera
    setup_camera(
        &mut commands,
        camera_settings.distance,
        camera_settings.speed,
    );

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground plane for reference
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.2, 0.2),
            ..default()
        })),
    ));

    // Create nodes and edges
    create_graph_visualization(&mut commands, &mut meshes, &mut materials, &graph_data);

    // Store graph data as a resource for later use
    commands.insert_resource(graph_data);

    // Setup UI
    setup_ui(&mut commands);
}
