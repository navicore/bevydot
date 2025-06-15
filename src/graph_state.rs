use crate::events::{EventResult, GraphEvent};
use bevy::prelude::*;
use dotparser::{GraphData as ParserGraphData, NodeInfo};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Manages the current state of the graph based on events
#[derive(Resource)]
pub struct GraphState {
    /// The underlying graph structure
    graph: DiGraph<NodeInfo, ()>,
    /// Mapping from node IDs to graph indices
    node_map: HashMap<String, NodeIndex>,
    /// Whether we're currently in a batch update
    in_batch: bool,
    /// Events accumulated during batch
    batch_events: Vec<GraphEvent>,
}

impl GraphState {
    /// Creates a new empty graph state
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            in_batch: false,
            batch_events: Vec::new(),
        }
    }

    /// Processes a graph event and updates the state
    pub fn process_event(&mut self, event: GraphEvent) -> EventResult {
        // If in batch, accumulate events
        if self.in_batch && !matches!(event, GraphEvent::BatchEnd) {
            self.batch_events.push(event);
            return EventResult::Success;
        }

        match event {
            GraphEvent::AddNode { id, info } => {
                if self.node_map.contains_key(&id) {
                    EventResult::NodeExists
                } else {
                    let idx = self.graph.add_node(info.into());
                    self.node_map.insert(id, idx);
                    EventResult::Success
                }
            }

            GraphEvent::UpdateNode { id, info } => {
                if let Some(&idx) = self.node_map.get(&id) {
                    self.graph
                        .node_weight_mut(idx)
                        .map_or(EventResult::NodeNotFound, |node| {
                            *node = info.into();
                            EventResult::Success
                        })
                } else {
                    EventResult::NodeNotFound
                }
            }

            GraphEvent::RemoveNode { id } => {
                if let Some(idx) = self.node_map.remove(&id) {
                    self.graph.remove_node(idx);
                    // Note: petgraph automatically removes connected edges
                    EventResult::Success
                } else {
                    EventResult::NodeNotFound
                }
            }

            GraphEvent::AddEdge { from, to } => {
                match (self.node_map.get(&from), self.node_map.get(&to)) {
                    (Some(&from_idx), Some(&to_idx)) => {
                        // Check if edge already exists
                        if self.graph.find_edge(from_idx, to_idx).is_some() {
                            EventResult::EdgeExists
                        } else {
                            self.graph.add_edge(from_idx, to_idx, ());
                            EventResult::Success
                        }
                    }
                    _ => EventResult::NodeNotFound,
                }
            }

            GraphEvent::RemoveEdge { from, to } => {
                match (self.node_map.get(&from), self.node_map.get(&to)) {
                    (Some(&from_idx), Some(&to_idx)) => {
                        if let Some(edge) = self.graph.find_edge(from_idx, to_idx) {
                            self.graph.remove_edge(edge);
                            EventResult::Success
                        } else {
                            EventResult::EdgeNotFound
                        }
                    }
                    _ => EventResult::NodeNotFound,
                }
            }

            GraphEvent::Clear => {
                self.graph.clear();
                self.node_map.clear();
                EventResult::Success
            }

            GraphEvent::BatchStart => {
                self.in_batch = true;
                self.batch_events.clear();
                EventResult::Success
            }

            GraphEvent::BatchEnd => {
                self.in_batch = false;
                // Process all batched events
                let events = std::mem::take(&mut self.batch_events);
                for event in events {
                    self.process_event(event);
                }
                EventResult::Success
            }
        }
    }

    /// Processes multiple events
    pub fn process_events(&mut self, events: Vec<GraphEvent>) -> Vec<EventResult> {
        events.into_iter().map(|e| self.process_event(e)).collect()
    }

    /// Creates a new `ParserGraphData` by rebuilding the graph
    pub fn as_graph_data(&self) -> ParserGraphData {
        let mut new_graph = DiGraph::new();
        let mut new_map = HashMap::new();

        // Rebuild the graph
        for (id, &old_idx) in &self.node_map {
            if let Some(node_info) = self.graph.node_weight(old_idx) {
                let new_idx = new_graph.add_node(NodeInfo {
                    name: node_info.name.clone(),
                    node_type: node_info.node_type.clone(),
                    level: node_info.level,
                });
                new_map.insert(id.clone(), new_idx);
            }
        }

        // Copy edges
        for edge in self.graph.edge_indices() {
            if let Some((from, to)) = self.graph.edge_endpoints(edge) {
                // Find the corresponding new indices
                let from_id = self
                    .node_map
                    .iter()
                    .find(|&(_, &idx)| idx == from)
                    .map(|(id, _)| id);
                let to_id = self
                    .node_map
                    .iter()
                    .find(|&(_, &idx)| idx == to)
                    .map(|(id, _)| id);

                if let (Some(from_id), Some(to_id)) = (from_id, to_id) {
                    if let (Some(&new_from), Some(&new_to)) =
                        (new_map.get(from_id), new_map.get(to_id))
                    {
                        new_graph.add_edge(new_from, new_to, ());
                    }
                }
            }
        }

        ParserGraphData {
            graph: new_graph,
            node_map: new_map,
        }
    }

    /// Returns the number of nodes in the graph
    #[allow(dead_code)] // Used in tests
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of edges in the graph
    #[allow(dead_code)] // Used in tests
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Gets a node by ID
    #[allow(dead_code)] // For future use
    pub fn get_node(&self, id: &str) -> Option<&NodeInfo> {
        self.node_map
            .get(id)
            .and_then(|&idx| self.graph.node_weight(idx))
    }
}

impl Default for GraphState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventNodeInfo;
    use dotparser::NodeType;

    #[test]
    fn test_add_and_remove_nodes() {
        let mut state = GraphState::new();

        // Add a node
        let result = state.process_event(GraphEvent::AddNode {
            id: "A".to_string(),
            info: EventNodeInfo {
                name: "Node A".to_string(),
                node_type: NodeType::Default,
                level: 0,
            },
        });

        assert!(matches!(result, EventResult::Success));
        assert_eq!(state.node_count(), 1);

        // Try to add the same node again
        let result = state.process_event(GraphEvent::AddNode {
            id: "A".to_string(),
            info: EventNodeInfo {
                name: "Node A".to_string(),
                node_type: NodeType::Default,
                level: 0,
            },
        });

        assert!(matches!(result, EventResult::NodeExists));
        assert_eq!(state.node_count(), 1);

        // Remove the node
        let result = state.process_event(GraphEvent::RemoveNode {
            id: "A".to_string(),
        });

        assert!(matches!(result, EventResult::Success));
        assert_eq!(state.node_count(), 0);
    }

    #[test]
    fn test_batch_processing() {
        let mut state = GraphState::new();

        let events = vec![
            GraphEvent::BatchStart,
            GraphEvent::AddNode {
                id: "A".to_string(),
                info: EventNodeInfo {
                    name: "A".to_string(),
                    node_type: NodeType::Default,
                    level: 0,
                },
            },
            GraphEvent::AddNode {
                id: "B".to_string(),
                info: EventNodeInfo {
                    name: "B".to_string(),
                    node_type: NodeType::Default,
                    level: 0,
                },
            },
            GraphEvent::AddEdge {
                from: "A".to_string(),
                to: "B".to_string(),
            },
            GraphEvent::BatchEnd,
        ];

        state.process_events(events);

        assert_eq!(state.node_count(), 2);
        assert_eq!(state.edge_count(), 1);
    }
}
