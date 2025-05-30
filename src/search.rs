use crate::types::{GraphNode, NodeHighlight, SearchBox, SearchState};
use bevy::prelude::*;

pub fn setup_search_ui(commands: &mut Commands) {
    // Create search box (initially hidden)
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Percent(50.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        Visibility::Hidden,
        SearchBox,
    ));
}

pub fn toggle_search(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<SearchState>,
    mut search_box_query: Query<&mut Visibility, With<SearchBox>>,
) {
    if keyboard_input.just_pressed(KeyCode::Slash) && !search_state.active {
        // Activate search
        search_state.active = true;
        search_state.query.clear();
        search_state.matching_nodes.clear();

        if let Ok(mut visibility) = search_box_query.single_mut() {
            *visibility = Visibility::Visible;
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) && search_state.active {
        // Deactivate search
        search_state.active = false;
        search_state.query.clear();
        search_state.matching_nodes.clear();
        // Don't clear selected_node here - let fly_to_selected_node handle it once

        if let Ok(mut visibility) = search_box_query.single_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn handle_search_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<SearchState>,
    mut search_box_query: Query<&mut Text, With<SearchBox>>,
    node_query: Query<(Entity, &GraphNode, &GlobalTransform)>,
    mut commands: Commands,
) {
    if !search_state.active {
        return;
    }

    // Check for letter keys
    for (key, ch) in [
        (KeyCode::KeyA, 'a'),
        (KeyCode::KeyB, 'b'),
        (KeyCode::KeyC, 'c'),
        (KeyCode::KeyD, 'd'),
        (KeyCode::KeyE, 'e'),
        (KeyCode::KeyF, 'f'),
        (KeyCode::KeyG, 'g'),
        (KeyCode::KeyH, 'h'),
        (KeyCode::KeyI, 'i'),
        (KeyCode::KeyJ, 'j'),
        (KeyCode::KeyK, 'k'),
        (KeyCode::KeyL, 'l'),
        (KeyCode::KeyM, 'm'),
        (KeyCode::KeyN, 'n'),
        (KeyCode::KeyO, 'o'),
        (KeyCode::KeyP, 'p'),
        (KeyCode::KeyQ, 'q'),
        (KeyCode::KeyR, 'r'),
        (KeyCode::KeyS, 's'),
        (KeyCode::KeyT, 't'),
        (KeyCode::KeyU, 'u'),
        (KeyCode::KeyV, 'v'),
        (KeyCode::KeyW, 'w'),
        (KeyCode::KeyX, 'x'),
        (KeyCode::KeyY, 'y'),
        (KeyCode::KeyZ, 'z'),
        (KeyCode::Space, ' '),
    ] {
        if keyboard_input.just_pressed(key) {
            if keyboard_input.pressed(KeyCode::ShiftLeft)
                || keyboard_input.pressed(KeyCode::ShiftRight)
            {
                search_state.query.push(ch.to_ascii_uppercase());
            } else {
                search_state.query.push(ch);
            }
            break;
        }
    }

    // Handle backspace
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        search_state.query.pop();
    }

    // Update search box text
    if let Ok(mut text) = search_box_query.single_mut() {
        text.0 = format!("Search: {}_", search_state.query);
    }

    // Find matching nodes
    search_state.matching_nodes.clear();
    if !search_state.query.is_empty() {
        for (entity, node, _) in &node_query {
            if node
                .name
                .to_lowercase()
                .contains(&search_state.query.to_lowercase())
            {
                search_state.matching_nodes.push(entity);
            }
        }
    }

    // Select the first matching node
    search_state.selected_node = search_state.matching_nodes.first().copied();

    // Update highlighting
    for (entity, _, _) in &node_query {
        if search_state.matching_nodes.contains(&entity) {
            // Add highlight component if not present
            commands
                .entity(entity)
                .try_insert(NodeHighlight { fade_timer: 1.0 });
        } else {
            // Remove highlight if present
            commands.entity(entity).remove::<NodeHighlight>();
        }
    }
}

// Removed fly_to_selected_node - search now only highlights nodes

pub fn update_node_highlighting(
    mut commands: Commands,
    mut highlight_query: Query<(Entity, &mut NodeHighlight)>,
    time: Res<Time>,
    search_state: Res<SearchState>,
) {
    let delta = time.delta_secs();

    for (entity, mut highlight) in &mut highlight_query {
        // Don't fade if search is active
        if search_state.active {
            highlight.fade_timer = 1.0;
        } else {
            // Fade out over 20 seconds (10x slower)
            highlight.fade_timer -= delta * 0.05;

            if highlight.fade_timer <= 0.0 {
                commands.entity(entity).remove::<NodeHighlight>();
            }
        }
    }
}

pub fn apply_highlight_visuals(
    node_query: Query<(&MeshMaterial3d<StandardMaterial>, Option<&NodeHighlight>), With<GraphNode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (material, highlight) in &node_query {
        let material_handle = &material.0;
        if let Some(material) = materials.get_mut(material_handle) {
            if let Some(highlight) = highlight {
                // Apply highlight effect (emissive glow)
                let intensity = highlight.fade_timer;
                material.emissive =
                    LinearRgba::new(intensity * 0.5, intensity * 0.5, intensity * 0.0, 1.0);
            } else {
                // Remove highlight
                material.emissive = LinearRgba::BLACK;
            }
        }
    }
}
