use bevy::prelude::*;
use dotparser::GraphData as ParserGraphData;
use petgraph::graph::NodeIndex;

// Re-export types from dotparser for use in other modules
pub use dotparser::NodeType;

// Wrapper to add Bevy Resource capability to GraphData
#[derive(Resource)]
pub struct GraphData(pub ParserGraphData);

// Implement Deref for transparent access to the underlying GraphData
impl std::ops::Deref for GraphData {
    type Target = ParserGraphData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GraphData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct DotContent(pub String);

#[derive(Component)]
pub struct GraphNode {
    pub name: String,
    pub index: NodeIndex,
}

#[derive(Component)]
pub struct GraphEdge {
    pub from: NodeIndex,
    pub to: NodeIndex,
}

#[derive(Component)]
pub struct NodeLabel {
    pub node_entity: Entity,
}

#[derive(Component)]
pub struct LabelVisibilityIndicator;

#[derive(Component)]
pub struct SearchBox;

#[derive(Resource, Default)]
pub struct SearchState {
    pub active: bool,
    pub query: String,
    pub matching_nodes: Vec<Entity>,
    pub selected_node: Option<Entity>,
}

#[derive(Component)]
pub struct NodeHighlight {
    pub fade_timer: f32,
}

#[derive(Resource)]
pub struct CameraSettings {
    pub distance: f32,
    pub speed: f32,
}

#[derive(Resource)]
pub struct LabelSettings {
    pub visibility_distance: f32,
    pub show_all_labels: bool,
}

impl Default for LabelSettings {
    fn default() -> Self {
        Self {
            visibility_distance: 15.0,
            show_all_labels: false,
        }
    }
}
