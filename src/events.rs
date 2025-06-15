use dotparser::{NodeInfo, NodeType};
use std::fmt;

/// Simplified node information for events
#[derive(Debug, Clone)]
pub struct EventNodeInfo {
    pub name: String,
    pub node_type: NodeType,
    pub level: u32,
}

impl From<&NodeInfo> for EventNodeInfo {
    fn from(info: &NodeInfo) -> Self {
        Self {
            name: info.name.clone(),
            node_type: info.node_type.clone(),
            level: info.level,
        }
    }
}

impl From<EventNodeInfo> for NodeInfo {
    fn from(info: EventNodeInfo) -> Self {
        Self {
            name: info.name,
            node_type: info.node_type,
            level: info.level,
        }
    }
}

/// Events that can modify the graph structure
#[derive(Debug, Clone)]
pub enum GraphEvent {
    /// Add a new node to the graph
    AddNode { id: String, info: EventNodeInfo },

    /// Update an existing node's properties
    UpdateNode { id: String, info: EventNodeInfo },

    /// Remove a node from the graph
    RemoveNode { id: String },

    /// Add an edge between two nodes
    AddEdge { from: String, to: String },

    /// Remove an edge between two nodes
    RemoveEdge { from: String, to: String },

    /// Clear the entire graph
    Clear,

    /// Start of a batch of events (for optimization)
    BatchStart,

    /// End of a batch of events
    BatchEnd,
}

impl GraphEvent {
    /// Returns true if this event modifies node data
    pub fn affects_node(&self, node_id: &str) -> bool {
        match self {
            Self::AddNode { id, .. } | Self::UpdateNode { id, .. } | Self::RemoveNode { id } => {
                id == node_id
            }
            Self::AddEdge { from, to } | Self::RemoveEdge { from, to } => {
                from == node_id || to == node_id
            }
            Self::Clear => true,
            Self::BatchStart | Self::BatchEnd => false,
        }
    }
}

impl fmt::Display for GraphEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddNode { id, info } => write!(f, "AddNode({}: {})", id, info.name),
            Self::UpdateNode { id, info } => write!(f, "UpdateNode({}: {})", id, info.name),
            Self::RemoveNode { id } => write!(f, "RemoveNode({})", id),
            Self::AddEdge { from, to } => write!(f, "AddEdge({} -> {})", from, to),
            Self::RemoveEdge { from, to } => write!(f, "RemoveEdge({} -> {})", from, to),
            Self::Clear => write!(f, "Clear"),
            Self::BatchStart => write!(f, "BatchStart"),
            Self::BatchEnd => write!(f, "BatchEnd"),
        }
    }
}

/// Result of processing a graph event
#[derive(Debug)]
pub enum EventResult {
    /// Event was processed successfully
    Success,
    /// Node already exists (for AddNode)
    NodeExists,
    /// Node not found (for UpdateNode, RemoveNode, edges)
    NodeNotFound,
    /// Edge already exists
    EdgeExists,
    /// Edge not found
    EdgeNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotparser::NodeType;

    #[test]
    fn test_event_affects_node() {
        let event = GraphEvent::AddNode {
            id: "A".to_string(),
            info: EventNodeInfo {
                name: "Node A".to_string(),
                node_type: NodeType::Default,
                level: 0,
            },
        };

        assert!(event.affects_node("A"));
        assert!(!event.affects_node("B"));
    }

    #[test]
    fn test_edge_events_affect_both_nodes() {
        let event = GraphEvent::AddEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };

        assert!(event.affects_node("A"));
        assert!(event.affects_node("B"));
        assert!(!event.affects_node("C"));
    }
}
