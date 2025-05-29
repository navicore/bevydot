use bevy::prelude::*;
use clap::Parser;
use petgraph::graph::DiGraph;
use std::collections::HashMap;
use std::io::{self, IsTerminal, Read};

#[derive(Parser, Debug)]
#[command(author, version, about = "3D visualization of Graphviz dot files", long_about = None)]
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
    let dot_content = if let Some(filename) = args.file {
        std::fs::read_to_string(&filename).unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        })
    } else if !io::stdin().is_terminal() {
        // Read from stdin if it's piped
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
            eprintln!("Error reading from stdin: {}", e);
            std::process::exit(1);
        });
        buffer
    } else {
        eprintln!("Error: No input provided. Either specify a file or pipe data to stdin.");
        eprintln!("Usage: bevydot [FILE] or command | bevydot");
        std::process::exit(1);
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DotContent(dot_content))
        .insert_resource(CameraConfig {
            initial_distance: args.distance,
            speed: args.speed,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_controls, update_node_labels, exit_on_key))
        .run();
}

#[derive(Resource)]
struct DotContent(String);

#[derive(Resource)]
struct CameraConfig {
    initial_distance: f32,
    speed: f32,
}

#[derive(Clone, Debug)]
struct NodeInfo {
    label: String,
    node_type: NodeType,
    level: u32,
}

#[derive(Clone, Debug, PartialEq)]
enum NodeType {
    Organization,
    LineOfBusiness,
    Site,
    Team,
    User,
    Default,
}

#[derive(Resource)]
struct GraphData {
    graph: DiGraph<NodeInfo, ()>,
    //node_map: HashMap<String, NodeIndex>,
}

fn spawn_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    from: Vec3,
    to: Vec3,
) {
    let direction = to - from;
    let distance = direction.length();
    let midpoint = from + direction * 0.5;

    // Calculate rotation to align cylinder with edge direction
    let up = Vec3::Y;
    let rotation = Quat::from_rotation_arc(up, direction.normalize());

    // Create a cylinder to represent the edge
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.02, distance))),
        MeshMaterial3d(material),
        Transform::from_translation(midpoint).with_rotation(rotation),
    ));
}

fn parse_dot_file(content: &str) -> GraphData {
    // For now, let's use a simple parser that extracts basic info
    let mut graph = DiGraph::new();
    let mut node_map = HashMap::new();
    let mut node_attributes = HashMap::new();

    // Parse nodes with attributes
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let trimmed = line.trim();

        // Parse node definitions with attributes
        if trimmed.contains('[') && trimmed.contains(']') && !trimmed.contains("->") {
            if let Some(node_end) = trimmed.find('[') {
                let node_id = trimmed[..node_end].trim().trim_matches('"');

                // Extract attributes
                let attrs_str = &trimmed[node_end + 1..trimmed.rfind(']').unwrap_or(trimmed.len())];
                let mut node_type = NodeType::Default;
                let mut level = 0u32;

                // Parse attributes
                for attr in attrs_str.split(',') {
                    let parts: Vec<&str> = attr.split('=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"');

                        match key {
                            "type" => {
                                node_type = match value {
                                    "organization" => NodeType::Organization,
                                    "lob" => NodeType::LineOfBusiness,
                                    "site" => NodeType::Site,
                                    "team" => NodeType::Team,
                                    "user" => NodeType::User,
                                    _ => NodeType::Default,
                                };
                            }
                            "level" => {
                                level = value.parse().unwrap_or(0);
                            }
                            _ => {}
                        }
                    }
                }

                node_attributes.insert(node_id.to_string(), (node_type, level));
            }
        }
    }

    // Parse edges and create nodes
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.contains("->") {
            // Remove comments
            let edge_line = if let Some(comment_pos) = trimmed.find("//") {
                &trimmed[..comment_pos]
            } else {
                trimmed
            };

            let parts: Vec<&str> = edge_line.split("->").collect();
            if parts.len() >= 2 {
                let from = parts[0].trim().trim_matches('"');
                let to_part = parts[1].trim();
                let to = if let Some(bracket_pos) = to_part.find('[') {
                    to_part[..bracket_pos]
                        .trim()
                        .trim_matches('"')
                        .trim_end_matches(';')
                } else {
                    to_part.trim_end_matches(';').trim().trim_matches('"')
                };

                // Ensure nodes exist
                let from_idx = *node_map.entry(from.to_string()).or_insert_with(|| {
                    let (node_type, level) = node_attributes
                        .get(from)
                        .cloned()
                        .unwrap_or((NodeType::Default, 0));
                    graph.add_node(NodeInfo {
                        label: from.to_string(),
                        node_type,
                        level,
                    })
                });

                let to_idx = *node_map.entry(to.to_string()).or_insert_with(|| {
                    let (node_type, level) = node_attributes
                        .get(to)
                        .cloned()
                        .unwrap_or((NodeType::Default, 0));
                    graph.add_node(NodeInfo {
                        label: to.to_string(),
                        node_type,
                        level,
                    })
                });

                graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    //GraphData { graph, node_map }
    GraphData { graph }
}

fn get_node_appearance(node_type: &NodeType) -> (Color, f32) {
    // Returns (color, size_multiplier)
    match node_type {
        NodeType::Organization => (Color::srgb(0.8, 0.2, 0.2), 1.5), // Red, large
        NodeType::LineOfBusiness => (Color::srgb(0.8, 0.5, 0.2), 1.2), // Orange
        NodeType::Site => (Color::srgb(0.2, 0.6, 0.8), 1.0),         // Blue
        NodeType::Team => (Color::srgb(0.2, 0.8, 0.5), 0.8),         // Green
        NodeType::User => (Color::srgb(0.6, 0.4, 0.8), 0.6),         // Purple, small
        NodeType::Default => (Color::srgb(0.5, 0.5, 0.5), 0.7),      // Gray
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dot_content: Res<DotContent>,
    camera_config: Res<CameraConfig>,
) {
    // Parse the dot content
    let graph_data = parse_dot_file(&dot_content.0);

    // 3D Camera - positioned to see the hierarchy
    let controller = CameraController {
        distance: camera_config.initial_distance,
        speed: camera_config.speed,
        ..Default::default()
    };

    // Calculate initial position based on orbit angles and distance
    let horizontal_distance = controller.distance * controller.pitch_angle.cos();
    let x = horizontal_distance * controller.orbit_angle.cos();
    let y = controller.distance * controller.pitch_angle.sin() + controller.look_at.y;
    let z = horizontal_distance * controller.orbit_angle.sin();

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(x, y, z).looking_at(controller.look_at, Vec3::Y),
        controller,
    ));

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

    // Create 3D nodes from the graph with hierarchical layout
    let mut level_counts = HashMap::new();
    let mut level_indices = HashMap::new();

    // Count nodes at each level
    for node_idx in graph_data.graph.node_indices() {
        let node_info = &graph_data.graph[node_idx];
        *level_counts.entry(node_info.level).or_insert(0) += 1;
    }

    // Create nodes with proper positioning
    for node_idx in graph_data.graph.node_indices() {
        let node_info = &graph_data.graph[node_idx];
        let (color, size_mult) = get_node_appearance(&node_info.node_type);

        // Get current index at this level
        let level_idx = level_indices.entry(node_info.level).or_insert(0);
        let count_at_level = level_counts[&node_info.level];

        // Calculate position with hierarchical layout
        let level_radius = 5.0 + (node_info.level as f32 * 2.0);
        let angle = 2.0 * std::f32::consts::PI * (*level_idx as f32) / count_at_level as f32;
        let x = level_radius * angle.cos();
        let z = level_radius * angle.sin();
        let y = node_info.level as f32 * 2.0; // Vertical spacing by level

        *level_idx += 1;

        // Create material for this node type
        let node_material = materials.add(StandardMaterial {
            base_color: color,
            ..default()
        });

        // Create mesh based on node type
        let mesh = match node_info.node_type {
            NodeType::Organization => meshes.add(Cuboid::new(1.0, 1.0, 1.0)), // Cube
            NodeType::LineOfBusiness => meshes.add(Cylinder::new(0.5, 1.0)),  // Cylinder
            NodeType::Site => meshes.add(Torus::new(0.3, 0.5)),               // Torus
            NodeType::Team => meshes.add(Sphere::new(0.5)),                   // Sphere
            NodeType::User => meshes.add(Capsule3d::new(0.3, 0.4)),           // Capsule
            NodeType::Default => meshes.add(Sphere::new(0.5)),
        };

        // Spawn node with appropriate shape
        let node_entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(node_material),
                Transform::from_xyz(x, y, z).with_scale(Vec3::splat(size_mult)),
                GraphNode,
            ))
            .id();

        // Store the label in the Name component
        commands
            .entity(node_entity)
            .insert(Name::new(node_info.label.clone()));

        // Create a UI text element for this node
        let label_entity = commands
            .spawn((
                Text::new(node_info.label.clone()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                NodeLabel {
                    offset: Vec3::new(0.0, 1.0, 0.0),
                },
            ))
            .id();

        // Link the label to the node
        commands
            .entity(node_entity)
            .insert(LabelEntity(label_entity));
    }

    // Create edges between nodes
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7),
        unlit: true,
        ..default()
    });

    // Reset level indices for edge position calculation
    level_indices.clear();

    // Get node positions for edge creation
    let mut node_positions = HashMap::new();
    for node_idx in graph_data.graph.node_indices() {
        let node_info = &graph_data.graph[node_idx];
        let level_idx = level_indices.entry(node_info.level).or_insert(0);
        let count_at_level = level_counts[&node_info.level];

        let level_radius = 5.0 + (node_info.level as f32 * 2.0);
        let angle = 2.0 * std::f32::consts::PI * (*level_idx as f32) / count_at_level as f32;
        let x = level_radius * angle.cos();
        let z = level_radius * angle.sin();
        let y = node_info.level as f32 * 2.0;

        *level_idx += 1;
        node_positions.insert(node_idx, Vec3::new(x, y, z));
    }

    // Create edges
    for edge in graph_data.graph.edge_indices() {
        if let Some((from_idx, to_idx)) = graph_data.graph.edge_endpoints(edge) {
            if let (Some(&from_pos), Some(&to_pos)) =
                (node_positions.get(&from_idx), node_positions.get(&to_idx))
            {
                spawn_edge(
                    &mut commands,
                    &mut meshes,
                    edge_material.clone(),
                    from_pos,
                    to_pos,
                );
            }
        }
    }

    // Store graph data as a resource for later use
    commands.insert_resource(graph_data);

    // Add control instructions
    commands.spawn((
        Text::new("Controls:\nArrows: Move\nShift+Arrows: Rotate\n+/- : Zoom\nESC/Q: Exit"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

#[derive(Component)]
struct CameraController {
    speed: f32,
    rotation_speed: f32,
    look_at: Vec3,
    distance: f32,
    orbit_angle: f32,
    pitch_angle: f32,
}

#[derive(Component)]
struct GraphNode;

#[derive(Component)]
struct NodeLabel {
    offset: Vec3,
}

#[derive(Component)]
struct LabelEntity(Entity);

impl Default for CameraController {
    fn default() -> Self {
        Self {
            speed: 5.0,
            rotation_speed: 1.0,
            look_at: Vec3::new(0.0, 4.0, 0.0), // Center of hierarchy
            distance: 25.0,
            orbit_angle: std::f32::consts::PI / 4.0, // 45 degrees
            pitch_angle: std::f32::consts::PI / 6.0, // 30 degrees
        }
    }
}

fn camera_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
) {
    for (mut transform, mut controller) in &mut query {
        let shift = keyboard_input.pressed(KeyCode::ShiftLeft)
            || keyboard_input.pressed(KeyCode::ShiftRight);

        // Zoom controls - multiple options for Mac compatibility
        if keyboard_input.pressed(KeyCode::PageUp)
            || keyboard_input.pressed(KeyCode::Equal)  // Plus key
            || keyboard_input.pressed(KeyCode::NumpadAdd)
        {
            controller.distance =
                (controller.distance - controller.speed * time.delta_secs()).max(5.0);
        }
        if keyboard_input.pressed(KeyCode::PageDown)
            || keyboard_input.pressed(KeyCode::Minus)
            || keyboard_input.pressed(KeyCode::NumpadSubtract)
        {
            controller.distance =
                (controller.distance + controller.speed * time.delta_secs()).min(50.0);
        }

        if shift {
            // Rotation mode - orbit around the look_at point
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                controller.orbit_angle += controller.rotation_speed * time.delta_secs();
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                controller.orbit_angle -= controller.rotation_speed * time.delta_secs();
            }
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                controller.pitch_angle = (controller.pitch_angle
                    - controller.rotation_speed * time.delta_secs())
                .clamp(-std::f32::consts::PI / 3.0, std::f32::consts::PI / 3.0);
                // Limit to +/- 60 degrees
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                controller.pitch_angle = (controller.pitch_angle
                    + controller.rotation_speed * time.delta_secs())
                .clamp(-std::f32::consts::PI / 3.0, std::f32::consts::PI / 3.0);
            }

            // Update camera position based on orbit angles
            let horizontal_distance = controller.distance * controller.pitch_angle.cos();
            let x = horizontal_distance * controller.orbit_angle.cos();
            let y = controller.distance * controller.pitch_angle.sin() + controller.look_at.y;
            let z = horizontal_distance * controller.orbit_angle.sin();

            transform.translation = controller.look_at + Vec3::new(x, y, z);
            transform.look_at(controller.look_at, Vec3::Y);
        } else {
            // Movement mode - move the look_at point
            let mut movement = Vec3::ZERO;
            let forward = transform.forward();
            let right = transform.right();

            if keyboard_input.pressed(KeyCode::ArrowUp) {
                movement += Vec3::new(forward.x, 0.0, forward.z).normalize();
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                movement -= Vec3::new(forward.x, 0.0, forward.z).normalize();
            }
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                movement -= Vec3::new(right.x, 0.0, right.z).normalize();
            }
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                movement += Vec3::new(right.x, 0.0, right.z).normalize();
            }

            if movement.length() > 0.0 {
                movement = movement.normalize() * controller.speed * time.delta_secs();
                controller.look_at += movement;
                transform.translation += movement;
            }
        }

        // Always update camera position after zoom changes
        if keyboard_input.pressed(KeyCode::PageUp)
            || keyboard_input.pressed(KeyCode::PageDown)
            || keyboard_input.pressed(KeyCode::Equal)
            || keyboard_input.pressed(KeyCode::Minus)
            || keyboard_input.pressed(KeyCode::NumpadAdd)
            || keyboard_input.pressed(KeyCode::NumpadSubtract)
        {
            let horizontal_distance = controller.distance * controller.pitch_angle.cos();
            let x = horizontal_distance * controller.orbit_angle.cos();
            let y = controller.distance * controller.pitch_angle.sin() + controller.look_at.y;
            let z = horizontal_distance * controller.orbit_angle.sin();

            transform.translation = controller.look_at + Vec3::new(x, y, z);
            transform.look_at(controller.look_at, Vec3::Y);
        }
    }
}

fn update_node_labels(
    nodes: Query<(&GlobalTransform, &LabelEntity), With<GraphNode>>,
    mut labels: Query<(&mut Node, &NodeLabel)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    if let Ok((camera, camera_transform)) = camera_query.single() {
        for (node_transform, label_entity) in &nodes {
            if let Ok((mut style, label)) = labels.get_mut(label_entity.0) {
                // Project 3D position to screen space
                let world_position = node_transform.translation() + label.offset;

                if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_position) {
                    style.left = Val::Px(screen_pos.x);
                    style.top = Val::Px(screen_pos.y);
                }
            }
        }
    }
}

fn exit_on_key(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph() {
        let dot_content = r#"
            digraph G {
                A -> B;
                B -> C;
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        // Should have 3 nodes
        assert_eq!(graph_data.graph.node_count(), 3);
        // Should have 2 edges
        assert_eq!(graph_data.graph.edge_count(), 2);
    }

    #[test]
    fn test_parse_node_with_attributes() {
        let dot_content = r#"
            digraph G {
                "CEO" [type="organization", level="3"];
                "Manager" [type="team", level="1"];
                "CEO" -> "Manager";
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        // Check nodes were created with correct attributes
        let ceo_node = graph_data
            .graph
            .node_indices()
            .find(|&idx| graph_data.graph[idx].label == "CEO")
            .expect("CEO node not found");

        assert_eq!(graph_data.graph[ceo_node].node_type, NodeType::Organization);
        assert_eq!(graph_data.graph[ceo_node].level, 3);

        let manager_node = graph_data
            .graph
            .node_indices()
            .find(|&idx| graph_data.graph[idx].label == "Manager")
            .expect("Manager node not found");

        assert_eq!(graph_data.graph[manager_node].node_type, NodeType::Team);
        assert_eq!(graph_data.graph[manager_node].level, 1);
    }

    #[test]
    fn test_parse_nodes_without_attributes() {
        let dot_content = r#"
            digraph G {
                "NodeA";
                "NodeB";
                "NodeA" -> "NodeB";
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        // Nodes without attributes should have defaults
        for node_idx in graph_data.graph.node_indices() {
            let node = &graph_data.graph[node_idx];
            assert_eq!(node.node_type, NodeType::Default);
            assert_eq!(node.level, 0);
        }
    }

    #[test]
    fn test_parse_edge_with_style() {
        let dot_content = r#"
            digraph G {
                A -> B [style="dashed"];
                B -> C;
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        // Should parse edges even with style attributes
        assert_eq!(graph_data.graph.edge_count(), 2);
    }

    #[test]
    fn test_node_type_parsing() {
        let test_cases = vec![
            ("organization", NodeType::Organization),
            ("lob", NodeType::LineOfBusiness),
            ("site", NodeType::Site),
            ("team", NodeType::Team),
            ("user", NodeType::User),
            ("unknown", NodeType::Default),
        ];

        for (type_str, expected_type) in test_cases {
            let dot_content = format!(
                "digraph G {{\n  \"TestNode\" [type=\"{}\"];\n  \"TestNode\" -> \"Dummy\";\n}}",
                type_str
            );

            let graph_data = parse_dot_file(&dot_content);

            let test_node = graph_data
                .graph
                .node_indices()
                .find(|&idx| graph_data.graph[idx].label == "TestNode")
                .expect("TestNode not found");
            assert_eq!(graph_data.graph[test_node].node_type, expected_type);
        }
    }

    #[test]
    fn test_quoted_node_names() {
        let dot_content = r#"
            digraph G {
                "Node with spaces" [type="team"];
                "Another Node" [type="user"];
                "Node with spaces" -> "Another Node";
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        // Should handle quoted node names properly
        assert_eq!(graph_data.graph.node_count(), 2);
        assert_eq!(graph_data.graph.edge_count(), 1);

        // Check that quotes were removed from labels
        let labels: Vec<String> = graph_data
            .graph
            .node_indices()
            .map(|idx| graph_data.graph[idx].label.clone())
            .collect();

        assert!(labels.contains(&"Node with spaces".to_string()));
        assert!(labels.contains(&"Another Node".to_string()));
    }

    #[test]
    fn test_get_node_appearance() {
        // Test color and size for each node type
        let test_cases = vec![
            (NodeType::Organization, 1.5),
            (NodeType::LineOfBusiness, 1.2),
            (NodeType::Site, 1.0),
            (NodeType::Team, 0.8),
            (NodeType::User, 0.6),
            (NodeType::Default, 0.7),
        ];

        for (node_type, expected_size) in test_cases {
            let (color, size) = get_node_appearance(&node_type);
            assert_eq!(size, expected_size);
            // Just verify we got a color (specific values might change)
            assert!(color.to_srgba().red >= 0.0);
        }
    }

    #[test]
    fn test_complex_graph_parsing() {
        let dot_content = r#"
            digraph ComplexGraph {
                // Comments should be ignored
                "Root" [type="organization", level="2"];
                "Child1" [type="team", level="1"];
                "Child2" [type="team", level="1"];
                "Leaf1" [type="user", level="0"];

                "Root" -> "Child1";
                "Root" -> "Child2";
                "Child1" -> "Leaf1";
                "Child2" -> "Leaf1"; // Multiple parents
            }
        "#;

        let graph_data = parse_dot_file(dot_content);

        assert_eq!(graph_data.graph.node_count(), 4);
        assert_eq!(graph_data.graph.edge_count(), 4);

        // Verify node attributes were parsed correctly
        let root = graph_data
            .graph
            .node_indices()
            .find(|&idx| graph_data.graph[idx].label == "Root")
            .unwrap();
        assert_eq!(graph_data.graph[root].level, 2);
    }
}
