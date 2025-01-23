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
    @location(6) texture        : i32, 
    @location(7) id             : i32, 
};

////////////////////////////////////////////////////////////////////////////////
// VERTEX -> FRAGMENT OUTPUT
////////////////////////////////////////////////////////////////////////////////

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color               : vec4<f32>,
    @location(1) uv                  : vec2<f32>,
    @location(2) texture            : i32,
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
    out.texture = in.texture; // Pass texture index

    return out;
}


////////////////////////////////////////////////////////////////////////////////
// TEXTURE BIND GROUP
////////////////////////////////////////////////////////////////////////////////

// DIGIT ATLAS
@group(2) @binding(0)
var digit_atlas_texture: texture_2d<f32>;
@group(2) @binding(1)
var digit_atlas_sampler: sampler;

// ARROW IMG
@group(2) @binding(2)
var arrow_texture: texture_2d<f32>;
@group(2) @binding(3)
var arrow_sampler: sampler;

// 'X' IMG
@group(2) @binding(4)
var x_texture: texture_2d<f32>;
@group(2) @binding(5)
var x_sampler: sampler;

// 'X' IMG
@group(2) @binding(6)
var destination_texture: texture_2d<f32>;
@group(2) @binding(7)
var destination_sampler: sampler;

////////////////////////////////////////////////////////////////////////////////
// FRAGMENT SHADER
////////////////////////////////////////////////////////////////////////////////
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // DIGIT ATLAS
    if (in.texture >= 0i && in.texture < 10i) {
        let digits_per_row = 10u; // Number of digits in the atlas
        let digit_width = 1.0 / f32(digits_per_row);
        let adjusted_uv = vec2<f32>(
            in.uv.x * digit_width + (f32(in.texture) * digit_width),
            in.uv.y
        );

        let tex_color = textureSample(digit_atlas_texture, digit_atlas_sampler, adjusted_uv);

        // Optional: Discard pixels with zero alpha to optimize rendering
        if (tex_color.a * in.color.a == 0.0) {
            discard;
        }

        // Multiply the texture color by the instance color, preserving alpha
        return tex_color * in.color;
    } 
    
    // ARROW IMG
    if (in.texture == -1i) {
        let tex_color = textureSample(arrow_texture, arrow_sampler, in.uv);

        if (tex_color.a * in.color.a == 0.0) {
            discard;
        }

        return tex_color * in.color;
    } 
    
    // 'X' IMG
    if (in.texture == -2i) {
        let tex_color = textureSample(x_texture, x_sampler, in.uv);

        if (tex_color.a * in.color.a == 0.0) {
            discard;
        }

        return tex_color * in.color;
    } 
    
    // DESTINATION IMG
    if(in.texture == -3i) {
        let tex_color = textureSample(destination_texture, destination_sampler, in.uv);

        if (tex_color.a * in.color.a == 0.0) {
            discard;
        }

        return tex_color * in.color;
    } 
    
    // GRID LINES
    return in.color;
}