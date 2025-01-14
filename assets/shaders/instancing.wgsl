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
    @location(0) position       : vec3<f32>,
    @location(2) uv             : vec2<f32>,
    @location(3) pos_scale      : vec4<f32>, 
    @location(4) rotation       : vec4<f32>,
    @location(5) color          : vec4<f32>,
    @location(6) digit          : f32, // New field for digit index
};

////////////////////////////////////////////////////////////////////////////////
// VERTEX -> FRAGMENT OUTPUT
////////////////////////////////////////////////////////////////////////////////

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color               : vec4<f32>,
    @location(1) uv                  : vec2<f32>,
    @location(2) digit               : f32, // Pass digit index to fragment shader
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

    // Rotate and transform the position
    let rotated = rotate_by_quat(in.position, in.rotation);
    let local_pos = rotated * in.pos_scale.w + in.pos_scale.xyz;

    // Transform by the camera's view-projection matrix
    out.clip_position = camera_data.view_proj * vec4<f32>(local_pos, 1.0);

    // Pass color and UV to the fragment shader
    out.color = in.color;
    out.uv = in.uv;
    out.digit = in.digit; // Pass digit index

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
    if (in.digit >= 0f && in.digit < 10f) { // Using u32 for digit
        let digits_per_row = 10u; // Number of digits in the atlas
        let digit_width = 1.0 / f32(digits_per_row);
        let adjusted_uv = vec2<f32>(
            in.uv.x * digit_width + (f32(in.digit) * digit_width),
            in.uv.y
        );

        let tex_color = textureSample(my_texture, my_sampler, adjusted_uv);

        // Optional: Discard pixels with zero alpha to optimize rendering
        if (tex_color.a * in.color.a == 0.0) {
            discard;
        }

        // Multiply the texture color by the instance color, preserving alpha
        return tex_color * in.color;
    } else {
        // If not using texture, return the instance color with its alpha
        return in.color;
    }
}
