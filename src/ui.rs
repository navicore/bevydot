use crate::types::{GraphNode, LabelSettings, LabelVisibilityIndicator, NodeLabel, SearchState};
use bevy::prelude::*;

pub fn setup_ui(commands: &mut Commands) {
    // Add control instructions
    commands.spawn((
        Text::new("Controls:\nArrows: Move\nShift+Arrows: Rotate\n+/- : Zoom\nL: Show all labels\n/: Search nodes\nESC: Close search\nQ: Exit"),
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

    // Add label visibility indicator
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 0.0, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        LabelVisibilityIndicator,
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
            Visibility::Hidden, // Start hidden, will be shown by update system if in range
        ));
    }
}

pub fn toggle_label_visibility(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut label_settings: ResMut<LabelSettings>,
    mut indicator_query: Query<&mut Text, With<LabelVisibilityIndicator>>,
    search_state: Res<SearchState>,
) {
    // Don't toggle labels when searching
    if search_state.active {
        return;
    }
    // Toggle show all labels with 'L' key
    if keyboard_input.pressed(KeyCode::KeyL) {
        label_settings.show_all_labels = true;
        if let Ok(mut text) = indicator_query.single_mut() {
            text.0 = "Showing all labels".to_string();
        }
    } else {
        label_settings.show_all_labels = false;
        if let Ok(mut text) = indicator_query.single_mut() {
            text.0 = String::new();
        }
    }
}

pub fn update_node_label_positions(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    node_query: Query<&GlobalTransform, With<GraphNode>>,
    mut label_query: Query<(&mut Node, &mut Visibility, &mut TextColor, &NodeLabel)>,
    label_settings: Res<LabelSettings>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    for (mut node_style, mut visibility, mut text_color, label) in &mut label_query {
        let Ok(node_transform) = node_query.get(label.node_entity) else {
            continue;
        };

        // Calculate distance from camera to node
        let distance = camera_transform
            .translation()
            .distance(node_transform.translation());

        // Show label if within distance threshold or if show_all_labels is true
        if label_settings.show_all_labels || distance <= label_settings.visibility_distance {
            *visibility = Visibility::Visible;

            // Fade labels based on distance (closer = more opaque)
            let fade_start = label_settings.visibility_distance * 0.7;
            let alpha = if distance < fade_start {
                1.0
            } else {
                1.0 - ((distance - fade_start) / (label_settings.visibility_distance - fade_start))
            };

            text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0));
        } else {
            *visibility = Visibility::Hidden;
        }

        // Project 3D position to screen coordinates
        if let Ok(viewport_position) =
            camera.world_to_viewport(camera_transform, node_transform.translation())
        {
            node_style.left = Val::Px(viewport_position.x);
            node_style.top = Val::Px(viewport_position.y);
        }
    }
}
