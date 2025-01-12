////////////////////////////////////////////////////////////////////////////////
// CAMERA UNIFORM (View + Projection)
// 
// By default, Bevy’s standard mesh pipeline provides a view bind group at @group(0).
// If you're rolling your own, you can define a CameraData uniform yourself
// with a single matrix (view_proj). Or you might rely on the existing "view" bind group 
// from `MeshPipeline`. 
//
// For illustrative purposes, this snippet shows an explicit uniform with a single matrix.
////////////////////////////////////////////////////////////////////////////////

struct CameraViewProj {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera_data: CameraViewProj;

////////////////////////////////////////////////////////////////////////////////
// PER-VERTEX + INSTANCE INPUT
// 
// The mesh vertex has:
//   - position (vec3)
//   - uv (vec2)
// And the instance attributes at @location(3..5):
//   - pos_scale (xyz = translation, w = uniform scale)
//   - rotation (quat as xyzw)
//   - color (rgba)
////////////////////////////////////////////////////////////////////////////////

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(2) uv       : vec2<f32>,
    @location(3) pos_scale: vec4<f32>, 
    @location(4) rotation : vec4<f32>,
    @location(5) color    : vec4<f32>,
};

////////////////////////////////////////////////////////////////////////////////
// VERTEX -> FRAGMENT OUTPUT
////////////////////////////////////////////////////////////////////////////////

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color              : vec4<f32>,
    @location(1) uv                 : vec2<f32>,
};

////////////////////////////////////////////////////////////////////////////////
// HELPER: ROTATION BY QUAT
////////////////////////////////////////////////////////////////////////////////

fn rotate_by_quat(pos: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let q_xyz = vec3<f32>(q.x, q.y, q.z);
    let t = 2.0 * cross(q_xyz, pos);
    return pos + q.w * t + cross(q_xyz, t);
}

////////////////////////////////////////////////////////////////////////////////
// VERTEX SHADER
////////////////////////////////////////////////////////////////////////////////

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // 1) rotate the mesh local position by the instance quaternion
    let rotated = rotate_by_quat(in.position, in.rotation);
    // 2) apply scale (pos_scale.w) + translation (pos_scale.xyz)
    let local_pos = rotated * in.pos_scale.w + in.pos_scale.xyz;

    // 3) transform by the camera's combined view+proj matrix
    out.clip_position = camera_data.view_proj * vec4<f32>(local_pos, 1.0);

    // pass color + uv to the fragment
    out.color = in.color;
    out.uv = in.uv;

    return out;
}

////////////////////////////////////////////////////////////////////////////////
// TEXTURE BIND GROUP
// 
// We assume you added a third bind group layout in your pipeline descriptor:
//   descriptor.layout = Some(vec![view_layout, mesh_layout, texture_layout])
// So these declarations match @group(2).
////////////////////////////////////////////////////////////////////////////////

@group(2) @binding(0)
var my_texture: texture_2d<f32>;

@group(2) @binding(1)
var my_sampler: sampler;

////////////////////////////////////////////////////////////////////////////////
// FRAGMENT SHADER
// 
// We sample the texture with the mesh uv, then multiply by instance color.
////////////////////////////////////////////////////////////////////////////////

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // sample from the digit texture
    let tex_color = textureSample(my_texture, my_sampler, in.uv);
    // multiply by the instance color to tint or set alpha
    return tex_color * in.color;
}
