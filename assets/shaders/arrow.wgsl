#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

// Define the vertex input structure
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) pos_scale: vec4<f32>,
    @location(2) color: vec4<f32>,
    // @location(3) i_rot: f32,
};

// Define the vertex output structure
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, // Position in clip space
    @location(0) color: vec4<f32>, // Color passed to the fragment shader
};

// Vertex shader
@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let position = vertex.position * vertex.pos_scale.w + vertex.pos_scale.xyz;
    var out: VertexOutput;
    // NOTE: Passing 0 as the instance_index to get_world_from_local() is a hack
    // for this example as the instance_index builtin would map to the wrong
    // index in the Mesh array. This index could be passed in via another
    // uniform instead but it's unnecessary for the example.
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0u),
        vec4<f32>(position, 1.0)
    );

    out.color = vertex.color;
    return out;
}

// Fragment shader
@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Use the instance color for the fragment
    return input.color;
}
