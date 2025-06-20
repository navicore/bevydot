use super::{GraphEventSource, SourceError};
use crate::events::{EventNodeInfo, GraphEvent};
use dotparser::plantuml;

/// Source for `PlantUML` format diagrams
pub struct PlantUMLSource {
    content: String,
}

impl PlantUMLSource {
    /// Creates a new `PlantUML` source from content
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Creates a new `PlantUML` source from a string slice
    pub fn from_content(content: &str) -> Self {
        Self::new(content.to_string())
    }
}

impl GraphEventSource for PlantUMLSource {
    fn source_name(&self) -> &'static str {
        "PlantUML"
    }

    fn events(&self) -> Result<Vec<GraphEvent>, SourceError> {
        // Parse the PlantUML content
        let dotparser_events = plantuml::parse(&self.content).map_err(SourceError::ParseError)?;

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
                dotparser::GraphEvent::AddNode {
                    id,
                    label,
                    node_type,
                    properties,
                } => {
                    // Convert to our EventNodeInfo
                    let info = EventNodeInfo {
                        name: label.unwrap_or_else(|| id.clone()),
                        node_type: match node_type {
                            dotparser::NodeType::Custom(t) => Some(t),
                            dotparser::NodeType::Actor { actor_type } => {
                                Some(format!("actor:{actor_type}"))
                            }
                            dotparser::NodeType::DataStore => Some("database".to_string()),
                            dotparser::NodeType::Process => Some("process".to_string()),
                            dotparser::NodeType::External => Some("external".to_string()),
                            _ => None,
                        },
                        level: match properties.position {
                            Some(dotparser::Position::Sequential { .. }) => 0, // All sequence participants at same level
                            Some(dotparser::Position::Layer { level }) => level,
                            _ => 1,
                        },
                    };

                    events.push(GraphEvent::AddNode { id, info });
                }
                dotparser::GraphEvent::AddEdge {
                    from,
                    to,
                    edge_type,
                    label,
                    ..
                } => {
                    // Extract edge properties based on edge type
                    let (edge_type_str, sequence_num) = match &edge_type {
                        dotparser::EdgeType::Message {
                            message_type,
                            sequence,
                        } => {
                            // Convert MessageType to string manually
                            let type_str = match message_type {
                                dotparser::MessageType::Synchronous => "sync",
                                dotparser::MessageType::Asynchronous => "async",
                                dotparser::MessageType::Return => "return",
                                dotparser::MessageType::Create => "create",
                                dotparser::MessageType::Destroy => "destroy",
                            };
                            (Some(type_str.to_string()), *sequence)
                        }
                        _ => (None, None),
                    };

                    // Preserve rich edge information from PlantUML
                    let edge_info = crate::events::EventEdgeInfo {
                        label,
                        edge_type: edge_type_str,
                        sequence: sequence_num,
                    };

                    events.push(GraphEvent::AddRichEdge {
                        from,
                        to,
                        info: edge_info,
                    });
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

    #[test]
    fn test_simple_plantuml_to_events() {
        let plantuml_content = r"
            @startuml
            actor User
            participant Server
            User -> Server: Request
            Server --> User: Response
            @enduml
        ";

        let source = PlantUMLSource::from_content(plantuml_content);
        let events = source.events().unwrap();

        // Should have: BatchStart, 2 AddNode, 2 AddEdge, BatchEnd
        assert!(events.len() >= 5);

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
            .filter(|e| matches!(e, GraphEvent::AddRichEdge { .. }))
            .count();

        assert_eq!(node_count, 2);
        assert_eq!(edge_count, 2);
    }
}
