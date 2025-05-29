# BevyDot Examples

This directory contains example Graphviz dot files that demonstrate different visualization scenarios with bevydot.

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