use bevy::prelude::*;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Organization,
    LineOfBusiness,
    Site,
    Team,
    User,
    Default,
}

impl NodeType {
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "organization" | "org" => Self::Organization,
            "lob" | "lineofbusiness" | "line_of_business" => Self::LineOfBusiness,
            "site" => Self::Site,
            "team" => Self::Team,
            "user" => Self::User,
            _ => Self::Default,
        }
    }
}

#[derive(Debug)]
pub struct NodeInfo {
    pub name: String,
    pub node_type: NodeType,
    pub level: u32,
}

#[derive(Resource)]
pub struct GraphData {
    pub graph: DiGraph<NodeInfo, ()>,
    #[allow(dead_code)]
    pub node_map: HashMap<String, NodeIndex>,
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
