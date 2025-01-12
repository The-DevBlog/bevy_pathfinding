use allocator::MeshAllocator;
use bevy::{
    asset::embedded_asset,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    image::*,
    pbr::*,
    prelude::*,
    render::{
        extract_component::*, mesh::*, render_asset::RenderAssets, render_phase::*,
        render_resource::*, renderer::RenderDevice, view::ExtractedView, *,
    },
};
use bytemuck::{Pod, Zeroable};
use extract_resource::{ExtractResource, ExtractResourcePlugin};
use image::ImageFormat;
use sync_world::MainEntity;
use texture::GpuImage;

use super::resources::DebugOptions;

const DIGIT_ATLAS: &[u8] = include_bytes!("../../assets/digits/digit_atlas.png");
const SHADER_PATH: &str = "shader_digit.wgsl";

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CustomShaderPlugin);
    }
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

struct CustomShaderPlugin;

impl Plugin for CustomShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            // MaterialPlugin::<CustomMaterial>::default(),
        ))
        .init_resource::<Digits>()
        .add_systems(Startup, load_digit_texture_atlas);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<Assets<Shader>>()
            .add_systems(ExtractSchedule, sync_data_from_main_app.run_if(run_once))
            .add_systems(
                Render,
                (
                    // queue_digit_bind_groups.in_set(RenderSet::QueueMeshes),
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );

        embedded_asset!(app, "instancing.wgsl");
        embedded_asset!(app, "instancing3.wgsl");
        embedded_asset!(app, "shader_digit.wgsl");
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

pub fn sync_data_from_main_app(mut cmds: Commands, world: ResMut<MainWorld>) {
    let Some(dbg) = world.get_resource::<DebugOptions>() else { return; };
    
    cmds.insert_resource(dbg.clone());
    dbg.print("\nsync_data() start");

    if let Some(digits) = world.get_resource::<Digits>() {
        cmds.insert_resource(digits.clone());
    }

    dbg.print("sync_data() end");
}

#[derive(Default, Resource, Clone, Deref, ExtractResource, Reflect)]
pub struct Digits(pub [Handle<Image>; 10]);

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec3,
    pub scale: f32,
    pub rotation: [f32; 4],
    pub color: [f32; 4],
}

fn load_digit_texture_atlas(mut images: ResMut<Assets<Image>>, mut digits: ResMut<Digits>, dbg: Res<DebugOptions>) {
    dbg.print("\nload_digit_texture_atlas() start");

    // Decode the image
    let image = image::load_from_memory_with_format(DIGIT_ATLAS, ImageFormat::Png)
        .expect("Failed to load digit image");
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let digit_width = width / 10;

    // Extract each digit as a separate texture
    for idx in 0..10 {
        let x_start = idx * digit_width;
        let cropped_digit_data =
            image::imageops::crop_imm(&rgba_image, x_start, 0, digit_width, height)
                .to_image()
                .into_raw();

        let cropped_digit = Image {
            data: cropped_digit_data,
            texture_descriptor: TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: digit_width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor::default()),
            texture_view_descriptor: None,
            asset_usage: Default::default(),
        };

        digits.0[idx as usize] = images.add(cropped_digit);
    }

    dbg.print("load_digit_texture_atlas() end");
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
    mut views: Query<(Entity, &ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    for (view_entity, view, msaa) in &mut views {
        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

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
                extra_index: PhaseItemExtraIndex::NONE,
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
    // texture_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = { world.resource::<MeshPipeline>().clone() };

        // Load the embedded shader by its virtual path
        let asset_server = world.resource::<AssetServer>();
        // let shader: Handle<Shader> =
        //     asset_server.load("embedded://bevy_rts_pathfinding/debug/instancing3.wgsl");
        let shader: Handle<Shader> =
            asset_server.load("embedded://bevy_rts_pathfinding/debug/instancing.wgsl");

        // Create a bind group layout for { texture, sampler }.
        let render_device = world.resource::<RenderDevice>();
        // let texture_layout = render_device.create_bind_group_layout(
        //     Some("digit_texture_layout"),
        //     &[
        //         // texture
        //         BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: BindingType::Texture {
        //                 multisampled: false,
        //                 sample_type: TextureSampleType::Float { filterable: true },
        //                 view_dimension: TextureViewDimension::D2,
        //             },
        //             count: None,
        //         },
        //         // sampler
        //         BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: ShaderStages::FRAGMENT,
        //             ty: BindingType::Sampler(SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ],
        // );

        CustomPipeline {
            shader,
            mesh_pipeline,
            // texture_layout,
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

        // descriptor.layout.push(self.texture_layout.clone());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            // shader locations 0-2 are taken up by Position, Normal and UV attributes
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3, // pos_scale
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size(),
                    shader_location: 4, // rotation
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x4.size() * 2,
                    shader_location: 5, // color
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
    // SetDigitTextureBindGroup<2>,
    DrawMeshInstanced,
);

// struct SetDigitTextureBindGroup<const I: usize>;

// impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetDigitTextureBindGroup<I> {
//     type Param = ();
//     type ViewQuery = ();
//     // This expects you to store something like `DigitBindGroup { bind_group: BindGroup }` on the item
//     type ItemQuery = Read<DigitBindGroup>;

//     fn render<'w>(
//         _item: &P,
//         _view: (),
//         digit_bind_group: Option<&'w DigitBindGroup>,
//         _param: SystemParamItem<'w, '_, Self::Param>,
//         pass: &mut TrackedRenderPass<'w>,
//     ) -> RenderCommandResult {
//         let Some(digit_bind_group) = digit_bind_group else {
//             return RenderCommandResult::Skip;
//         };
//         pass.set_bind_group(I, &digit_bind_group.bind_group, &[]);
//         RenderCommandResult::Success
//     }
// }

#[derive(Component)]
struct DigitBindGroup {
    bind_group: BindGroup,
}

// fn queue_digit_bind_groups(
//     mut commands: Commands,
//     digits: Res<Digits>, // your array of 10 handles
//     // digits: Extract<Res<Digits>>, // your array of 10 handles
//     gpu_images: Res<RenderAssets<GpuImage>>,
//     pipeline: Res<CustomPipeline>,
//     render_device: Res<RenderDevice>,
//     // Entities that need a texture bind group, but don't have it yet
//     q_entities: Query<Entity, (With<InstanceMaterialData>, Without<DigitBindGroup>)>,
// ) {
//     // For example, always pick digit 0. Or pick whichever you want
//     let handle = &digits.0[0];

//     if let Some(gpu_image) = gpu_images.get(handle) {
//         for entity in &q_entities {
//             // render_device.create_bind_group_layout
//             let bind_group = render_device.create_bind_group(
//                 Some("digit bind group"),
//                 &pipeline.texture_layout, // match your pipeline
//                 &[
//                     BindGroupEntry {
//                         binding: 0,
//                         resource: BindingResource::TextureView(&gpu_image.texture_view),
//                     },
//                     BindGroupEntry {
//                         binding: 1,
//                         resource: BindingResource::Sampler(&gpu_image.sampler),
//                     },
//                 ],
//             );

//             commands
//                 .entity(entity)
//                 .insert(DigitBindGroup { bind_group });
//         }
//     }
// }

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

// This struct defines the data that will be passed to your shader
// #[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
// pub struct CustomMaterial {
//     #[uniform(0)]
//     pub color: LinearRgba,
//     #[texture(1)]
//     #[sampler(2)]
//     pub color_texture: Option<Handle<Image>>,
//     pub alpha_mode: AlphaMode,
// }

// /// The Material trait is very configurable, but comes with sensible defaults for all methods.
// /// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
// impl Material for CustomMaterial {
//     fn fragment_shader() -> ShaderRef {
//         SHADER_PATH.into()
//     }

//     fn alpha_mode(&self) -> AlphaMode {
//         self.alpha_mode
//     }
// }
