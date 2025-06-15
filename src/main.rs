#![allow(clippy::cast_precision_loss)] // We accept precision loss for f32 conversions
#![allow(clippy::needless_pass_by_value)] // Bevy systems require owned Res parameters
#![allow(clippy::multiple_crate_versions)] // Bevy dependencies have multiple versions

use bevy::prelude::*;
use clap::Parser;
use std::io::{self, IsTerminal, Read};

mod camera;
mod events;
mod graph_state;
mod search;
mod sources;
mod types;
mod ui;
mod visualization;

use camera::{setup_camera, CameraPlugin};
use graph_state::GraphState;
use search::{
    apply_highlight_visuals, handle_search_input, setup_search_ui, toggle_search,
    update_node_highlighting,
};
use sources::dot::DotSource;
use sources::plantuml::PlantUMLSource;
use sources::{detect_format, GraphEventSource};
use types::{CameraSettings, DotContent, LabelSettings, SearchState};
use ui::{create_node_labels, setup_ui, toggle_label_visibility, update_node_label_positions};
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

    /// Label visibility distance
    #[arg(short = 'v', long, default_value = "15.0")]
    label_distance: f32,
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
        .insert_resource(LabelSettings {
            visibility_distance: args.label_distance,
            show_all_labels: false,
        })
        .insert_resource(SearchState::default())
        .add_plugins(CameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_label_visibility)
        .add_systems(Update, toggle_search)
        .add_systems(Update, handle_search_input)
        .add_systems(Update, update_node_highlighting)
        .add_systems(Update, apply_highlight_visuals)
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
    // Detect format and create appropriate source
    let format = detect_format(&dot_content.0).unwrap_or_else(|| {
        eprintln!("Warning: Could not detect diagram format, assuming DOT");
        "dot"
    });

    let events = if format == "plantuml" {
        let source = PlantUMLSource::from_content(&dot_content.0);
        source.events().expect("Failed to parse PlantUML file")
    } else {
        let source = DotSource::from_content(&dot_content.0);
        source.events().expect("Failed to parse DOT file")
    };

    let mut graph_state = GraphState::new();
    graph_state.process_events(events);

    // Convert to GraphData for compatibility
    let graph_data = types::GraphData(graph_state.as_graph_data());

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
    setup_search_ui(&mut commands);
}
