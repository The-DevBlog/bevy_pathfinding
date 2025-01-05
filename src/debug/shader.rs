//! A shader that renders a mesh multiple times in one draw call.
//!
//! Bevy will automatically batch and instance your meshes assuming you use the same
//! `Handle<Material>` and `Handle<Mesh>` for all of your instances.
//!
//! This example is intended for advanced users and shows how to make a custom instancing
//! implementation using bevy's low level rendering api.
//! It's generally recommended to try the built-in instancing before going with this approach.

use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        event::EventRegistry,
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{
            allocator::MeshAllocator, MeshVertexBufferLayoutRef, RenderMesh, RenderMeshBufferInfo,
        },
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::{ExtractedView, NoFrustumCulling},
        Render, RenderApp, RenderSet,
    },
};
use bevy_render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    RenderPlugin,
};
use bytemuck::{Pod, Zeroable};

/// This example uses a shader source file from the assets subdirectory
// const SHADER_ASSET_PATH: &str = "arrow.wgsl";
// const SHADER_ASSET_PATH: &str = "instancing.wgsl";

const SHADER: &str = r#"
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

// Define the vertex input structure
struct Vertex {
    @location(0) position: vec3<f32>, 
    @location(3) pos_scale: vec4<f32>,
    @location(4) color: vec4<f32>,
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
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShaderPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
    //     InstanceMaterialData(
    //         (1..=100)
    //             .flat_map(|x| (1..=100).map(move |y| (x as f32 / 10.0, y as f32 / 10.0)))
    //             .map(|(x, y)| InstanceData {
    //                 position: Vec3::new(x * 10.0 - 5.0, y * 10.0 - 5.0, 0.0),
    //                 scale: 1.0,
    //                 color: [0.2, 0.6, 0.8, 1.0],
    //             })
    //             .collect(),
    //     ),
    //     // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
    //     // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
    //     // instanced cubes will be culled.
    //     // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
    //     // instancing, and that is not taken into account with the built-in frustum culling.
    //     // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
    //     // component to avoid incorrect culling.
    //     NoFrustumCulling,
    // ));

    // camera
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(0.0, 0.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     // We need this component because we use `draw_indexed` and `draw`
    //     // instead of `draw_indirect_indexed` and `draw_indirect` in
    //     // `DrawMeshInstanced::render`.
    //     NoIndirectDrawing,
    // ));
}

#[derive(Component, Deref)]
pub struct InstanceMaterialData(pub Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstanceMaterialData>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<Assets<Shader>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    // pub rotation: Quat,
    pub color: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(Entity, &ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    for (view_entity, view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(&pipeline_cache, &custom_pipeline, key, &mesh.layout)
                .unwrap();
            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                distance: rangefinder.distance_translation(&mesh_instance.translation),
                batch_range: 0..1,
                // extra_index: PhaseItemExtraIndex::None,
                extra_index: PhaseItemExtraIndex(0),
            });
        }
    }
}

#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

// impl FromWorld for CustomPipeline {
//     fn from_world(world: &mut World) -> Self {
//         println!("from_world");
//         let mesh_pipeline = world.resource::<MeshPipeline>();
//         // Grab the shader handle we embedded at startup
//         // let embedded_shader = world.resource::<MyEmbeddedShader>();

//         // Grab the `Assets<Shader>` resource, so we can insert our WGSL
//         let mut shaders = world.resource_mut::<Assets<Shader>>();
//         let embedded_shader_handle = shaders.add(Shader::from_wgsl(SHADER, "dummy_path.wgsl"));

//         CustomPipeline {
//             // shader: embedded_shader.0.clone(), // Use the embedded shader handle
//             // shader: world.load_asset(SHADER_ASSET_PATH), // add inline shader here
//             shader: embedded_shader_handle, // add inline shader here
//             mesh_pipeline: mesh_pipeline.clone(),
//         }
//     }
// }

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        // Step 1: Immutable borrow in a limited scope
        let mesh_pipeline = {
            let mesh_pipeline_ref = world.resource::<MeshPipeline>();
            mesh_pipeline_ref.clone()
        };
        // `mesh_pipeline_ref` is dropped here when the scope ends

        // Step 2: Mutable borrow in a new scope
        let embedded_shader_handle = {
            let mut shaders = world.resource_mut::<Assets<Shader>>();
            shaders.add(Shader::from_wgsl(SHADER, "arrow.wgsl"))
        };

        CustomPipeline {
            // shader: world.load_asset(SHADER), // add inline shader here
            shader: embedded_shader_handle,
            mesh_pipeline,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // A borrow check workaround.
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed(
                    index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
                    vertex_buffer_slice.range.start as i32,
                    0..instance_buffer.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
