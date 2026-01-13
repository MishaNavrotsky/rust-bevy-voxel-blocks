use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraph, RenderLabel},
        render_resource::{
            UniformBuffer,
            binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    },
};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;

use crate::chunks_partition::{CHUNK_COUNT, CHUNK_SIZE, CHUNK_VOXELS_COUNT, VisibleChunks};

const SHADER_ASSET_PATH: &str = "shaders/voxel_gen.wgsl";

#[repr(C)]
#[derive(Clone, Copy, ShaderType, Pod, Zeroable, Debug)]
pub struct Globals {
    chunk_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType, Debug)]
pub struct ChunkCoord {
    coord: [i32; 3],
    global_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ShaderType, Debug)]
pub struct Vertex {
    coord: [f32; 4],
}

pub struct VoxelComputeGridPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct VoxelComputeNode;

struct VoxelComputeNodeImpl;

impl Node for VoxelComputeNodeImpl {
    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<VoxelComputeGridPipeline>();

        let Some(height_map_pipeline) =
            pipeline_cache.get_compute_pipeline(pipeline.height_map_pipeline)
        else {
            return Ok(());
        };

        let Some(vert_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.vert_pipeline)
        else {
            return Ok(());
        };

        let Some(bind_group) = world.get_resource::<VoxelComputeGridBindGroup>() else {
            return Ok(());
        };

        let mut pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("voxel_compute_pass"),
                    ..default()
                });

        pass.set_pipeline(height_map_pipeline);
        pass.set_bind_group(0, &bind_group.0, &[]);

        let h_wg = 4;
        let h_wg_per_axis = CHUNK_SIZE as u32 / h_wg;
        pass.dispatch_workgroups(
            h_wg_per_axis,
            h_wg_per_axis,
            CHUNK_COUNT as u32 * h_wg_per_axis,
        );

        pass.set_pipeline(vert_pipeline);
        pass.set_bind_group(0, &bind_group.0, &[]);

        let v_wg_x = 8;
        let v_wg_y = 8;

        let dispatch_x = (CHUNK_SIZE as u32 - 1 + v_wg_x - 1) / v_wg_x;
        let dispatch_y = (CHUNK_SIZE as u32 - 1 + v_wg_y - 1) / v_wg_y;
        let dispatch_z = CHUNK_COUNT as u32;
        pass.dispatch_workgroups(dispatch_x, dispatch_y, dispatch_z);

        Ok(())
    }
}

impl Plugin for VoxelComputeGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_voxel_compute_grid)
            .add_plugins(ExtractResourcePlugin::<VisibleChunks>::default())
            .add_plugins(ExtractResourcePlugin::<VoxelComputeGridImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_voxel_compute_grid_pipeline)
            .add_systems(Render, prepare_voxel_buffers.in_set(RenderSystems::Prepare))
            .add_systems(
                Render,
                prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
            );

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(VoxelComputeNode, VoxelComputeNodeImpl);
        graph.add_node_edge(VoxelComputeNode, CameraDriverLabel);
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct VoxelComputeGridImage {
    pub globals: Globals,
    pub chunks: Handle<ShaderStorageBuffer>,
    pub voxel_buffer: Handle<ShaderStorageBuffer>,
    pub vertecies_output: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
struct VoxelComputeGridBindGroup(BindGroup);

#[derive(Resource)]
pub struct VoxelComputeGridPipeline {
    bind_group_layout: BindGroupLayout,
    vert_pipeline: CachedComputePipelineId,
    height_map_pipeline: CachedComputePipelineId,
}

fn setup_voxel_compute_grid(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let chunk_count = CHUNK_COUNT * std::mem::size_of::<ChunkCoord>();
    let mut chunks_ssb = ShaderStorageBuffer::with_size(chunk_count, RenderAssetUsages::all());
    chunks_ssb.buffer_description.usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
    let chunks = buffers.add(chunks_ssb);

    let total_voxels = CHUNK_COUNT * CHUNK_VOXELS_COUNT;
    let buffer_size_bytes = total_voxels * std::mem::size_of::<f32>();
    let mut voxels_ssb =
        ShaderStorageBuffer::with_size(buffer_size_bytes, RenderAssetUsages::all());
    voxels_ssb.buffer_description.usage = BufferUsages::STORAGE | BufferUsages::COPY_DST;
    let voxels = buffers.add(voxels_ssb);

    let vertecies_count = CHUNK_COUNT * CHUNK_VOXELS_COUNT * std::mem::size_of::<Vertex>();
    let mut vertecies_output_ssb =
        ShaderStorageBuffer::with_size(vertecies_count, RenderAssetUsages::all());
    vertecies_output_ssb.buffer_description.usage =
        BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::VERTEX;
    let vertecies_output = buffers.add(vertecies_output_ssb);

    commands.insert_resource(VoxelComputeGridImage {
        globals: Globals {
            chunk_size: CHUNK_SIZE as u32,
        },
        chunks,
        voxel_buffer: voxels,
        vertecies_output,
    });
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<VoxelComputeGridPipeline>,
    image: Res<VoxelComputeGridImage>,
    render_device: Res<RenderDevice>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    queue: Res<RenderQueue>,
) {
    let Some(chunks_gpu) = buffers.get(&image.chunks) else {
        return;
    };

    let Some(voxels_gpu) = buffers.get(&image.voxel_buffer) else {
        return;
    };

    let Some(vertecies_gpu) = buffers.get(&image.vertecies_output) else {
        return;
    };

    let mut s = UniformBuffer::from(image.globals);
    s.write_buffer(&render_device, &queue);

    let bind_group = render_device.create_bind_group(
        Some("voxel_compute_bind_group"),
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((
            &s,
            chunks_gpu.buffer.as_entire_buffer_binding(),
            voxels_gpu.buffer.as_entire_buffer_binding(),
            vertecies_gpu.buffer.as_entire_buffer_binding(),
        )),
    );

    commands.insert_resource(VoxelComputeGridBindGroup(bind_group));
}

fn init_voxel_compute_grid_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let bind_group_layout = render_device.create_bind_group_layout(
        Some("VoxelComputeGridImage Layout"),
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<Globals>(false),
                storage_buffer_read_only::<ChunkCoord>(false),
                storage_buffer::<f32>(false),
                storage_buffer::<Vertex>(false),
            ),
        ),
    );

    let shader: Handle<Shader> = asset_server.load(SHADER_ASSET_PATH);
    let vert_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![bind_group_layout.clone()],
        shader: shader.clone(),
        entry_point: Some(Cow::from("generate_vertecies")),
        ..default()
    });
    let heeight_map_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("generate_height_map")),
        ..default()
    });

    commands.insert_resource(VoxelComputeGridPipeline {
        bind_group_layout,
        vert_pipeline,
        height_map_pipeline: heeight_map_pipeline,
    });
}

fn prepare_voxel_buffers(
    visible: Res<VisibleChunks>,
    image: Res<VoxelComputeGridImage>,
    mut gpu_buffers: ResMut<RenderAssets<GpuShaderStorageBuffer>>,
    render_queue: Res<RenderQueue>,
) {
    let Some(chunks_gpu) = gpu_buffers.get_mut(&image.chunks) else {
        return;
    };

    let mut chunk_data = Vec::with_capacity(visible.chunks.len());
    for c in &visible.chunks {
        chunk_data.push(ChunkCoord {
            coord: c.0.to_array(),
            global_index: c.1,
        });
    }

    render_queue.write_buffer(&chunks_gpu.buffer, 0, bytemuck::cast_slice(&chunk_data));
}
