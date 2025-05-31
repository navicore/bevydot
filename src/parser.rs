#![allow(clippy::cast_possible_truncation)] // Stack depth won't exceed u32::MAX

use crate::types::{GraphData, NodeInfo, NodeType};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

#[must_use]
pub fn parse_dot_file(content: &str) -> GraphData {
    let mut graph = DiGraph::new();
    let mut node_map = HashMap::new();
    let mut node_attributes = HashMap::new();

    // Check if this is a nested subgraph format (no edges)
    let has_edges = content.contains("->");

    if !has_edges && content.contains("subgraph") {
        // Parse nested subgraph format
        return parse_nested_subgraphs(content);
    }

    // Parse nodes with attributes (original edge-based format)
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let trimmed = line.trim();

        // Parse node definitions with attributes
        if trimmed.contains('[') && trimmed.contains(']') && !trimmed.contains("->") {
            if let Some(node_end) = trimmed.find('[') {
                let node_id = trimmed[..node_end].trim().trim_matches('"');

                // Extract attributes
                let attrs_str = &trimmed[node_end + 1..trimmed.rfind(']').unwrap_or(trimmed.len())];
                let mut node_type = NodeType::Default;
                let mut level = 0u32;

                // Parse attributes
                for attr in attrs_str.split(',') {
                    let parts: Vec<&str> = attr.split('=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let value = parts[1].trim().trim_matches('"');

                        match key {
                            "type" => node_type = NodeType::parse(value),
                            "level" => level = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                }

                node_attributes.insert(node_id.to_string(), (node_type, level));
            }
        }
    }

    // Parse edges and create nodes
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.contains("->") {
            // Remove comments
            let edge_line = trimmed
                .find("//")
                .map_or(trimmed, |comment_pos| &trimmed[..comment_pos]);

            let parts: Vec<&str> = edge_line.split("->").collect();
            if parts.len() >= 2 {
                let from = parts[0].trim().trim_matches('"');
                let to_part = parts[1].trim();
                let to = to_part.find('[').map_or_else(
                    || to_part.trim_end_matches(';').trim().trim_matches('"'),
                    |bracket_pos| {
                        to_part[..bracket_pos]
                            .trim()
                            .trim_matches('"')
                            .trim_end_matches(';')
                    },
                );

                // Ensure nodes exist
                let from_idx = *node_map.entry(from.to_string()).or_insert_with(|| {
                    let (node_type, level) = node_attributes
                        .get(from)
                        .cloned()
                        .unwrap_or((NodeType::Default, 0));
                    graph.add_node(NodeInfo {
                        name: from.to_string(),
                        node_type,
                        level,
                    })
                });

                let to_idx = *node_map.entry(to.to_string()).or_insert_with(|| {
                    let (node_type, level) = node_attributes
                        .get(to)
                        .cloned()
                        .unwrap_or((NodeType::Default, 0));
                    graph.add_node(NodeInfo {
                        name: to.to_string(),
                        node_type,
                        level,
                    })
                });

                graph.add_edge(from_idx, to_idx, ());
            }
        }
    }

    GraphData { graph, node_map }
}

fn parse_nested_subgraphs(content: &str) -> GraphData {
    let mut graph = DiGraph::new();
    let mut node_map = HashMap::new();
    let mut stack: Vec<(String, Option<petgraph::graph::NodeIndex>)> = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Parse subgraph clusters
        if line.starts_with("subgraph cluster_") {
            let cluster_name = line
                .strip_prefix("subgraph cluster_")
                .unwrap_or("")
                .trim_end_matches('{')
                .trim();

            // Look for the label in the next few lines
            let mut label = cluster_name.to_string();
            let level = stack.len() as u32;

            for next_line in lines.iter().skip(i + 1).take(9) {
                let next_line = next_line.trim();
                if next_line.starts_with("Label=") || next_line.starts_with("label=") {
                    label = next_line
                        .split('=')
                        .nth(1)
                        .unwrap_or(cluster_name)
                        .trim_matches('"')
                        .trim_matches(';')
                        .to_string();

                    // Extract meaningful name from label (after the colon if present)
                    if let Some(colon_pos) = label.find(':') {
                        label = label[colon_pos + 1..].trim().to_string();
                    }
                    break;
                }
            }

            // Determine node type based on label content
            let node_type = if label.to_lowercase().contains("tenant")
                || label.to_lowercase().contains("organization")
            {
                NodeType::Organization
            } else if label.to_lowercase().contains("contact center") {
                NodeType::LineOfBusiness
            } else if label.to_lowercase().contains("site") {
                NodeType::Site
            } else {
                NodeType::Default
            };

            // Create node for this cluster
            let node_idx = graph.add_node(NodeInfo {
                name: label.clone(),
                node_type,
                level,
            });

            node_map.insert(label, node_idx);

            // Connect to parent if exists
            if let Some((_, Some(parent_idx))) = stack.last() {
                graph.add_edge(*parent_idx, node_idx, ());
            }

            stack.push((cluster_name.to_string(), Some(node_idx)));
        }
        // Parse standalone nodes (like "supervisor32")
        else if line.contains('[') && line.contains("label=") && !line.contains("->") {
            if let Some(node_end) = line.find('[') {
                let node_id = line[..node_end].trim().trim_matches('"');

                // Extract label from attributes
                let label = line
                    .find("label=")
                    .and_then(|label_start| {
                        let label_part = &line[label_start + 6..];
                        label_part.find('"').and_then(|label_end| {
                            label_part[label_end + 1..].find('"').map(|second_quote| {
                                label_part[label_end + 1..label_end + 1 + second_quote]
                                    .replace("\\n", " ")
                                    .trim()
                                    .to_string()
                            })
                        })
                    })
                    .unwrap_or_else(|| node_id.to_string());

                let level = stack.len() as u32;
                let node_type = if label.to_lowercase().contains("supervisor") {
                    NodeType::Team
                } else {
                    NodeType::User
                };

                let node_idx = graph.add_node(NodeInfo {
                    name: label.clone(),
                    node_type,
                    level,
                });

                node_map.insert(label, node_idx);

                // Connect to parent if exists
                if let Some((_, Some(parent_idx))) = stack.last() {
                    graph.add_edge(*parent_idx, node_idx, ());
                }
            }
        }
        // Handle closing braces
        else if line.contains('}') && !stack.is_empty() {
            stack.pop();
        }

        i += 1;
    }

    GraphData { graph, node_map }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph() {
        let dot = r"
            digraph {
                A -> B;
                B -> C;
            }
        ";
        let graph_data = parse_dot_file(dot);
        assert_eq!(graph_data.graph.node_count(), 3);
        assert_eq!(graph_data.graph.edge_count(), 2);
    }

    #[test]
    fn test_node_type_parsing() {
        let test_cases = vec![
            ("organization", NodeType::Organization),
            ("org", NodeType::Organization),
            ("lob", NodeType::LineOfBusiness),
            ("lineofbusiness", NodeType::LineOfBusiness),
            ("line_of_business", NodeType::LineOfBusiness),
            ("site", NodeType::Site),
            ("team", NodeType::Team),
            ("user", NodeType::User),
            ("unknown", NodeType::Default),
        ];

        for (input, expected) in test_cases {
            assert_eq!(NodeType::parse(input), expected);
        }
    }

    #[test]
    fn test_parse_node_with_attributes() {
        let dot = r#"
            digraph {
                "Node1" [type="team", level="2"];
                "Node2" [type="user", level="1"];
                "Node1" -> "Node2";
            }
        "#;
        let graph_data = parse_dot_file(dot);

        let node1 = &graph_data.graph[graph_data.node_map["Node1"]];
        assert_eq!(node1.node_type, NodeType::Team);
        assert_eq!(node1.level, 2);
    }

    #[test]
    fn test_parse_edge_with_style() {
        let dot = r#"
            digraph {
                A -> B [style="dashed"];
                B -> C; // Comment
            }
        "#;
        let graph_data = parse_dot_file(dot);
        assert_eq!(graph_data.graph.edge_count(), 2);
    }

    #[test]
    fn test_quoted_node_names() {
        let dot = r#"
            digraph {
                "Node with spaces" -> "Another node";
                "Another node" -> SimpleNode;
            }
        "#;
        let graph_data = parse_dot_file(dot);
        assert_eq!(graph_data.graph.node_count(), 3);
        assert!(graph_data.node_map.contains_key("Node with spaces"));
    }

    #[test]
    fn test_parse_nodes_without_attributes() {
        let dot = r"
            digraph {
                NodeA;
                NodeB;
                NodeC;
                NodeA -> NodeB;
                NodeB -> NodeC;
            }
        ";
        let graph_data = parse_dot_file(dot);
        assert_eq!(graph_data.graph.node_count(), 3);

        for node in graph_data.graph.node_weights() {
            assert_eq!(node.node_type, NodeType::Default);
            assert_eq!(node.level, 0);
        }
    }

    #[test]
    fn test_nested_subgraph_parsing() {
        let dot = r#"
            digraph OrgHierarchy {
                subgraph cluster_tenant {
                    Label="Tenant: main_tenant";
                    subgraph cluster_org {
                        label="Organization: org1";
                        subgraph cluster_site {
                            label="Site: site1";
                            "user1" [label="User One"];
                        }
                    }
                }
            }
        "#;

        let graph_data = parse_dot_file(dot);

        // Should create 4 nodes: tenant, org, site, and user
        assert_eq!(graph_data.graph.node_count(), 4);
        // Should create 3 edges: tenant->org, org->site, site->user
        assert_eq!(graph_data.graph.edge_count(), 3);
    }

    #[test]
    fn test_complex_graph_parsing() {
        let dot = r#"
            digraph OrgChart {
                rankdir=TB;
                
                // Organization level
                "ACME Corp" [type="organization", level="3"];
                
                // Business units
                "Sales" [type="lob", level="2"];
                "Engineering" [type="lob", level="2"];
                
                // Sites
                "NYC Office" [type="site", level="1"];
                "SF Office" [type="site", level="1"];
                
                // Teams
                "Frontend Team" [type="team", level="1"];
                "Backend Team" [type="team", level="1"];
                
                // Connections
                "ACME Corp" -> "Sales";
                "ACME Corp" -> "Engineering";
                "Sales" -> "NYC Office";
                "Engineering" -> "SF Office";
                "Engineering" -> "Frontend Team";
                "Engineering" -> "Backend Team";
            }
        "#;

        let graph_data = parse_dot_file(dot);

        // Check node count
        assert_eq!(graph_data.graph.node_count(), 7);
        assert_eq!(graph_data.graph.edge_count(), 6);

        // Verify specific nodes
        let acme = &graph_data.graph[graph_data.node_map["ACME Corp"]];
        assert_eq!(acme.node_type, NodeType::Organization);
        assert_eq!(acme.level, 3);

        let frontend = &graph_data.graph[graph_data.node_map["Frontend Team"]];
        assert_eq!(frontend.node_type, NodeType::Team);
        assert_eq!(frontend.level, 1);
    }
}
