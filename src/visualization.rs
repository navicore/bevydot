use crate::types::{GraphData, GraphEdge, GraphNode};
use bevy::prelude::*;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

#[must_use]
pub fn get_node_appearance(node_type: Option<&str>) -> (Color, f32) {
    // Returns (color, size_multiplier)
    match node_type {
        // DOT diagram types
        Some("organization") => (Color::srgb(0.8, 0.2, 0.2), 1.5), // Red, large
        Some("line_of_business") => (Color::srgb(0.8, 0.5, 0.2), 1.2), // Orange
        Some("site") => (Color::srgb(0.2, 0.6, 0.8), 1.0),         // Blue
        Some("team") => (Color::srgb(0.2, 0.8, 0.5), 0.8),         // Green
        Some("user") => (Color::srgb(0.6, 0.4, 0.8), 0.6),         // Purple, small

        // PlantUML sequence diagram types
        Some("database") => (Color::srgb(0.2, 0.4, 0.7), 1.0), // Dark blue
        Some("actor:participant") => (Color::srgb(0.3, 0.7, 0.3), 0.8), // Green for regular participants
        Some(t) if t.starts_with("actor:") => (Color::srgb(0.9, 0.6, 0.2), 0.9), // Orange for actors
        Some("process") => (Color::srgb(0.7, 0.7, 0.2), 0.8),                    // Yellow
        Some("external") => (Color::srgb(0.5, 0.2, 0.7), 0.9),                   // Purple

        _ => (Color::srgb(0.5, 0.5, 0.5), 0.7), // Gray (default)
    }
}

pub fn create_graph_visualization(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    graph_data: &GraphData,
) -> HashMap<NodeIndex, Entity> {
    let mut node_entities = HashMap::new();
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
        let (color, size_mult) = get_node_appearance(node_info.node_type.as_deref());

        // Get current index at this level
        let level_idx = level_indices.entry(node_info.level).or_insert(0);
        let count_at_level = level_counts[&node_info.level];

        // Calculate position with hierarchical layout
        let level_radius = (node_info.level as f32).mul_add(2.0, 5.0);
        let angle = 2.0 * std::f32::consts::PI * (*level_idx as f32) / count_at_level as f32;
        let x = level_radius * angle.cos();
        let z = level_radius * angle.sin();
        let y = node_info.level as f32 * 2.0; // Vertical spacing by level

        *level_idx += 1;

        // Create material for this node type
        let node_material = materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::BLACK,
            ..default()
        });

        // Create mesh based on node type
        let mesh = match node_info.node_type.as_deref() {
            // DOT diagram shapes
            Some("organization") => meshes.add(Cuboid::new(1.0, 1.0, 1.0)), // Cube
            Some("line_of_business") => meshes.add(Cylinder::new(0.5, 1.0)), // Cylinder
            Some("site") => meshes.add(Torus::new(0.3, 0.5)),               // Torus
            Some("team") => meshes.add(Sphere::new(0.6)),                   // Sphere
            Some("user") => meshes.add(Capsule3d::new(0.3, 0.4)),           // Capsule

            // PlantUML sequence diagram shapes
            Some("database") => meshes.add(Cylinder::new(0.6, 0.8)), // Wide cylinder for DB
            Some("actor:participant") => meshes.add(Cuboid::new(0.8, 0.8, 0.8)), // Cube for services
            Some(t) if t.starts_with("actor:") => {
                // Actor as a humanoid shape (capsule)
                meshes.add(Capsule3d::new(0.4, 0.6))
            }
            Some("process") => meshes.add(Sphere::new(0.5)), // Sphere for process
            Some("external") => meshes.add(Torus::new(0.25, 0.5)), // Torus for external

            _ => meshes.add(Sphere::new(0.5)), // Default sphere
        };

        // Spawn node with appropriate shape
        let node_entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(node_material),
                Transform::from_xyz(x, y, z).with_scale(Vec3::splat(size_mult)),
                GraphNode {
                    name: node_info.name.clone(),
                    index: node_idx,
                },
                Name::new(node_info.name.clone()),
            ))
            .id();

        node_entities.insert(node_idx, node_entity);
    }

    // Create edges
    let edge_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4),
        ..default()
    });

    for edge in graph_data.graph.edge_indices() {
        if let Some((from_idx, to_idx)) = graph_data.graph.edge_endpoints(edge) {
            if let (Some(&from_entity), Some(&to_entity)) =
                (node_entities.get(&from_idx), node_entities.get(&to_idx))
            {
                let edge_info = graph_data.graph.edge_weight(edge);
                spawn_edge(
                    commands,
                    meshes,
                    materials,
                    edge_material.clone(),
                    from_entity,
                    to_entity,
                    from_idx,
                    to_idx,
                    edge_info,
                );
            }
        }
    }

    node_entities
}

#[allow(clippy::too_many_arguments)]
fn spawn_edge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    default_material: Handle<StandardMaterial>,
    _from_entity: Entity,
    _to_entity: Entity,
    from_idx: NodeIndex,
    to_idx: NodeIndex,
    edge_info: Option<&crate::graph_state::EdgeInfo>,
) {
    let (color, thickness) = edge_info.map_or_else(
        || (Color::srgb(0.4, 0.4, 0.4), 0.02), // Default gray
        |info| {
            match info.edge_type.as_deref() {
                Some("sync") => (Color::srgb(0.2, 0.4, 0.8), 0.03), // Blue, thick
                Some("async") => (Color::srgb(0.8, 0.4, 0.2), 0.02), // Orange, normal
                Some("return") => (Color::srgb(0.4, 0.8, 0.4), 0.015), // Green, thin
                _ => (Color::srgb(0.4, 0.4, 0.4), 0.02),            // Gray default
            }
        },
    );

    // Create material for this edge type
    let edge_material = if edge_info.is_some() {
        materials.add(StandardMaterial {
            base_color: color,
            emissive: LinearRgba::from(color) * 0.2, // Slight glow for sequence edges
            ..default()
        })
    } else {
        default_material
    };

    let edge_component = GraphEdge {
        from: from_idx,
        to: to_idx,
        label: edge_info.and_then(|info| info.label.clone()),
        edge_type: edge_info.and_then(|info| info.edge_type.clone()),
        sequence: edge_info.and_then(|info| info.sequence),
    };

    // Create main edge cylinder
    let edge_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cylinder::new(thickness, 1.0))),
            MeshMaterial3d(edge_material.clone()),
            Transform::default(),
            edge_component,
        ))
        .id();

    // Add arrow head for directional edges
    if edge_info.is_some() {
        commands.spawn((
            Mesh3d(meshes.add(Cone {
                radius: thickness * 3.0,
                height: thickness * 8.0,
            })),
            MeshMaterial3d(edge_material),
            Transform::default(),
            crate::types::EdgeArrowHead { edge: edge_entity },
        ));
    }
}

#[allow(clippy::type_complexity)]
pub fn update_edge_positions(
    node_query: Query<(&Transform, &GraphNode)>,
    mut edge_query: Query<(Entity, &mut Transform, &GraphEdge), Without<GraphNode>>,
    mut arrow_query: Query<
        (&mut Transform, &crate::types::EdgeArrowHead),
        (Without<GraphEdge>, Without<GraphNode>),
    >,
    _graph_data: Res<GraphData>,
) {
    // Create a map of node indices to positions
    let mut node_positions = HashMap::new();
    for (transform, graph_node) in &node_query {
        node_positions.insert(graph_node.index, transform.translation);
    }

    // Update edge positions
    for (edge_entity, mut edge_transform, graph_edge) in &mut edge_query {
        if let (Some(&from_pos), Some(&to_pos)) = (
            node_positions.get(&graph_edge.from),
            node_positions.get(&graph_edge.to),
        ) {
            let direction = to_pos - from_pos;
            let distance = direction.length();
            let midpoint = from_pos + direction * 0.5;

            // Calculate rotation to align cylinder with edge direction
            let up = Vec3::Y;
            let rotation = if direction.normalize().dot(up).abs() > 0.999 {
                // Edge is nearly vertical, use a different approach
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
            } else {
                Quat::from_rotation_arc(up, direction.normalize())
            };

            edge_transform.translation = midpoint;
            edge_transform.rotation = rotation;
            edge_transform.scale = Vec3::new(1.0, distance, 1.0);

            // Update arrow head position if this edge has one
            for (mut arrow_transform, arrow_head) in &mut arrow_query {
                if arrow_head.edge == edge_entity {
                    // Position arrow at the end of the edge, slightly before the target node
                    let arrow_offset = direction.normalize() * 0.5; // Offset from target
                    arrow_transform.translation = to_pos - arrow_offset;
                    arrow_transform.rotation = rotation;
                }
            }
        }
    }
}
