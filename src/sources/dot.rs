use super::{GraphEventSource, SourceError};
use crate::events::{EventNodeInfo, GraphEvent};
use dotparser::dot;
use std::collections::HashSet;

/// Source for DOT format diagrams
pub struct DotSource {
    content: String,
}

impl DotSource {
    /// Creates a new DOT source from content
    pub fn new(content: String) -> Self {
        Self { content }
    }
    
    /// Creates a new DOT source from a string slice
    pub fn from_str(content: &str) -> Self {
        Self::new(content.to_string())
    }
}

impl GraphEventSource for DotSource {
    fn source_name(&self) -> &'static str {
        "DOT"
    }
    
    fn events(&self) -> Result<Vec<GraphEvent>, SourceError> {
        // Parse the DOT content
        let graph_data = dot::parse(&self.content);
        
        let mut events = Vec::new();
        let mut seen_nodes = HashSet::new();
        
        // Start batch for efficiency
        events.push(GraphEvent::BatchStart);
        
        // First pass: collect all nodes
        for node_index in graph_data.graph.node_indices() {
            if let Some(node_info) = graph_data.graph.node_weight(node_index) {
                // Use the node's name as its ID
                let node_id = node_info.name.clone();
                
                // Handle duplicate names by appending index
                let final_id = if seen_nodes.contains(&node_id) {
                    let mut counter = 2;
                    let mut candidate = format!("{}_{}", node_id, counter);
                    while seen_nodes.contains(&candidate) {
                        counter += 1;
                        candidate = format!("{}_{}", node_id, counter);
                    }
                    candidate
                } else {
                    node_id
                };
                
                seen_nodes.insert(final_id.clone());
                
                events.push(GraphEvent::AddNode {
                    id: final_id,
                    info: EventNodeInfo::from(node_info),
                });
            }
        }
        
        // Second pass: add all edges
        for edge in graph_data.graph.edge_indices() {
            if let Some((from_idx, to_idx)) = graph_data.graph.edge_endpoints(edge) {
                // Get node names to use as IDs
                if let (Some(from_node), Some(to_node)) = (
                    graph_data.graph.node_weight(from_idx),
                    graph_data.graph.node_weight(to_idx),
                ) {
                    events.push(GraphEvent::AddEdge {
                        from: from_node.name.clone(),
                        to: to_node.name.clone(),
                    });
                }
            }
        }
        
        // End batch
        events.push(GraphEvent::BatchEnd);
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_dot_to_events() {
        let dot_content = r#"
            digraph {
                A -> B;
                B -> C;
            }
        "#;
        
        let source = DotSource::from_str(dot_content);
        let events = source.events().unwrap();
        
        // Should have: BatchStart, 3 AddNode, 2 AddEdge, BatchEnd
        assert_eq!(events.len(), 7);
        
        // Check batch markers
        assert!(matches!(events.first(), Some(GraphEvent::BatchStart)));
        assert!(matches!(events.last(), Some(GraphEvent::BatchEnd)));
        
        // Count event types
        let node_count = events.iter().filter(|e| matches!(e, GraphEvent::AddNode { .. })).count();
        let edge_count = events.iter().filter(|e| matches!(e, GraphEvent::AddEdge { .. })).count();
        
        assert_eq!(node_count, 3);
        assert_eq!(edge_count, 2);
    }
    
    #[test]
    fn test_duplicate_node_names() {
        let dot_content = r#"
            digraph {
                Server -> Database;
                Server -> Cache;
                "Server" -> Queue;
            }
        "#;
        
        let source = DotSource::from_str(dot_content);
        let events = source.events().unwrap();
        
        // Should handle duplicate "Server" nodes
        let node_events: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                GraphEvent::AddNode { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect();
        
        // Should have unique IDs
        let unique_ids: HashSet<_> = node_events.iter().cloned().collect();
        assert_eq!(node_events.len(), unique_ids.len());
    }
}