use super::{GraphEventSource, SourceError};
use crate::events::{EventNodeInfo, GraphEvent};
use dotparser::dot;

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
    pub fn from_content(content: &str) -> Self {
        Self::new(content.to_string())
    }
}

impl GraphEventSource for DotSource {
    fn source_name(&self) -> &'static str {
        "DOT"
    }

    fn events(&self) -> Result<Vec<GraphEvent>, SourceError> {
        // Parse the DOT content directly to events
        let dotparser_events = dot::parse(&self.content);
        
        // Convert dotparser events to our internal events
        let mut events = Vec::new();
        
        for event in dotparser_events {
            match event {
                dotparser::GraphEvent::BatchStart => {
                    events.push(GraphEvent::BatchStart);
                }
                dotparser::GraphEvent::BatchEnd => {
                    events.push(GraphEvent::BatchEnd);
                }
                dotparser::GraphEvent::AddNode { id, label, node_type, properties } => {
                    // Convert to our EventNodeInfo
                    let info = EventNodeInfo {
                        name: label.unwrap_or_else(|| id.clone()),
                        node_type: match node_type {
                            dotparser::NodeType::Custom(t) => Some(t),
                            _ => None,
                        },
                        level: match properties.position {
                            Some(dotparser::Position::Layer { level }) => level,
                            _ => 0,
                        },
                    };
                    
                    events.push(GraphEvent::AddNode { id, info });
                }
                dotparser::GraphEvent::AddEdge { from, to, .. } => {
                    events.push(GraphEvent::AddEdge { from, to });
                }
                _ => {
                    // Ignore other event types for now
                }
            }
        }
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_state::GraphState;
    use std::collections::HashSet;

    #[test]
    fn test_simple_dot_to_events() {
        let dot_content = r"
            digraph {
                A -> B;
                B -> C;
            }
        ";

        let source = DotSource::from_content(dot_content);
        let events = source.events().unwrap();

        // Should have: BatchStart, 3 AddNode, 2 AddEdge, BatchEnd
        assert_eq!(events.len(), 7);

        // Check batch markers
        assert!(matches!(events.first(), Some(GraphEvent::BatchStart)));
        assert!(matches!(events.last(), Some(GraphEvent::BatchEnd)));

        // Count event types
        let node_count = events
            .iter()
            .filter(|e| matches!(e, GraphEvent::AddNode { .. }))
            .count();
        let edge_count = events
            .iter()
            .filter(|e| matches!(e, GraphEvent::AddEdge { .. }))
            .count();

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

        let source = DotSource::from_content(dot_content);
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

    #[test]
    fn test_event_stream_with_attributes() {
        // Test that our event system properly handles node attributes
        let dot_content = r#"
            digraph {
                A [type="team", level="2"];
                B [type="user", level="1"];
                C [type="user", level="1"];
                A -> B;
                A -> C;
                B -> C;
            }
        "#;

        // Get graph via event stream
        let source = DotSource::from_content(dot_content);
        let events = source.events().unwrap();
        let mut state = GraphState::new();
        state.process_events(events);
        let event_graph = state.as_graph_data();

        // Verify structure
        assert_eq!(event_graph.graph.node_count(), 3);
        assert_eq!(event_graph.graph.edge_count(), 3);

        // Verify all nodes exist
        assert!(event_graph.node_map.contains_key("A"));
        assert!(event_graph.node_map.contains_key("B"));
        assert!(event_graph.node_map.contains_key("C"));

        // Check node properties
        let a_idx = event_graph.node_map["A"];
        let a_node = &event_graph.graph[a_idx];
        assert_eq!(a_node.name, "A");
        assert_eq!(a_node.node_type, Some("team".to_string()));
        assert_eq!(a_node.level, 2);

        let b_idx = event_graph.node_map["B"];
        let b_node = &event_graph.graph[b_idx];
        assert_eq!(b_node.name, "B");
        assert_eq!(b_node.node_type, Some("user".to_string()));
        assert_eq!(b_node.level, 1);
    }
}
