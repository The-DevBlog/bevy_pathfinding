```rust

FlowFieldProps {
    cell_radius: f32
    cell_diameter: f32
    grid: Vec<Vec<Cell>>
    offset: Vec3
    steering_map: HashMap<Entity, Vec3>
    units: Vec<Entity>
}

ParentFlowField {
    destination_cell: Cell
    flowfield_props: FlowFieldProps
}

DestinationFlowField {
    destination_cells: Vec<(Cell, bool)>
    destination_radius: f32
    flowfield_props: FlowFieldProps
}


```