use crate::types::{GraphNode, NodeLabel};
use bevy::prelude::*;

pub fn setup_ui(commands: &mut Commands) {
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

pub fn create_node_labels(
    mut commands: Commands,
    node_query: Query<(Entity, &GraphNode), Added<GraphNode>>,
) {
    for (node_entity, graph_node) in &node_query {
        // Create a UI text element for this node
        commands.spawn((
            Text::new(graph_node.name.clone()),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            NodeLabel { node_entity },
        ));
    }
}

pub fn update_node_label_positions(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<&GlobalTransform, With<GraphNode>>,
    mut label_query: Query<(&mut Node, &NodeLabel)>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for (mut node_style, label) in &mut label_query {
        let Ok(node_transform) = node_query.get(label.node_entity) else {
            continue;
        };

        // Project 3D position to screen coordinates
        if let Ok(viewport_position) =
            camera.world_to_viewport(camera_transform, node_transform.translation())
        {
            node_style.left = Val::Px(viewport_position.x);
            node_style.top = Val::Px(viewport_position.y);
        }
    }
}
