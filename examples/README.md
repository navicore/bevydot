# Dotspace Examples

This directory contains example Graphviz dot files that demonstrate different visualization scenarios with dotspace.

## Files

### hierarchy.dot
The original organizational hierarchy example showing:
- Different node types (organization, line of business, site, team, user)
- Hierarchical levels with vertical positioning
- Different shapes and colors for each type

### network_topology.dot
A network infrastructure diagram showing:
- Router and switch hierarchy
- Server and workstation connections
- Redundant connections (shown as dashed lines)

### software_architecture.dot
A software system architecture showing:
- Multi-tier application structure
- Service dependencies
- Database connections
- Cross-service communication

### project_dependencies.dot
A project dependency graph showing:
- Library dependencies
- External package usage
- Circular dependencies (shown as dashed)

### simple_graph.dot
A basic graph without custom attributes to test default rendering.

### org.dot
An organizational hierarchy using nested subgraphs instead of edges:
- Demonstrates containment relationships through subgraph nesting
- Shows tenant → organization → contact center → site → supervisor hierarchy
- No explicit edges - relationships are implied by nesting

### hybrid_architecture.dot
A complex microservices architecture combining both approaches:
- Subgraphs represent team ownership and logical grouping
- Edges show service dependencies and API calls
- Demonstrates how organizational structure and functional relationships can coexist
- Note: Currently parsed as edge-based only (subgraph structure is visual only)

### team_collaboration.dot
A simpler hybrid example showing:
- Team structure using subgraphs (Engineering, Design, Product)
- Collaboration patterns using directed edges
- Cross-team communication flows
- Note: Currently parsed as edge-based only (subgraph structure is visual only)

## Usage

Run any example with:

```bash
# Direct file
cargo run -- examples/hierarchy.dot

# With pipe
cat examples/network_topology.dot | cargo run

# With custom camera settings
cargo run -- -d 30 -s 8 examples/software_architecture.dot
```

## Parser Behavior

The dotspace parser currently handles two distinct formats:

1. **Edge-based graphs** (traditional dot format): Files containing `->` edges are parsed to create nodes and explicit connections
2. **Nested subgraph format**: Files with only subgraphs (no edges) are parsed to create hierarchical containment relationships

Files containing both edges and subgraphs (like `hybrid_architecture.dot`) are currently parsed as edge-based only. The subgraph structure provides visual grouping in standard Graphviz tools but doesn't create containment relationships in dotspace.

## Custom Attributes

The visualizer recognizes these node attributes:

- `type`: Determines shape and color
  - "organization" - Red cube (largest)
  - "lob" (line of business) - Orange cylinder
  - "site" - Blue torus
  - "team" - Green sphere
  - "user" - Purple capsule (smallest)
  
- `level`: Vertical positioning (0 = ground, higher = elevated)

Example:
```dot
"Node Name" [type="team", level="2"];
```