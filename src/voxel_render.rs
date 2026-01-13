use std::{borrow::Cow, sync::Arc};

use bevy::{
    asset::RenderAssetUsages,
    mesh::VertexBufferLayout,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        graph::CameraDriverLabel,
        render_asset::RenderAssets,
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{
            UniformBuffer,
            binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
        view::ViewTarget,
    },
};

use crate::{
    chunks_partition::{CHUNK_COUNT, CHUNK_VOXELS_COUNT},
    voxel_compute_grid::{Vertex, VoxelComputeGridImage},
};

const SHADER_ASSET_PATH: &str = "shaders/voxel.wgsl";

#[derive(Resource)]
pub struct VoxelRenderPipeline {
    pub pipeline_id: CachedRenderPipelineId,
    pub layout: BindGroupLayout,
}

fn init_voxel_render_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
) {
    let shader = asset_server.load(SHADER_ASSET_PATH);

    let vertex_layout = VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: vec![VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        }],
    };

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("voxel_render_pipeline".into()),
        layout: vec![], // no bind groups yet
        vertex: VertexState {
            shader: shader.clone(),
            entry_point: Some(Cow::from("vs_main")),
            buffers: vec![vertex_layout],
            ..Default::default()
        },
        fragment: Some(FragmentState {
            shader,
            entry_point: Some(Cow::from("fs_main")),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
            ..Default::default()
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            ..default()
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        push_constant_ranges: vec![],
        ..Default::default()
    });

    commands.insert_resource(VoxelRenderPipeline {
        pipeline_id,
        layout: render_device.create_bind_group_layout(None, &[]),
    });
}

struct VoxelDrawNode;

impl Node for VoxelDrawNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<VoxelRenderPipeline>();
        let buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        let image = world.resource::<VoxelComputeGridImage>();

        let Some(gpu_vertex_buffer) = buffers.get(&image.vertecies_output) else {
            return Ok(());
        };

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        for entity in world.iter_resources() {
            if let Some(view_target) = Arc::new(entity)::<ViewTarget>() {}
        }

        for view_target in world.query::<&ViewTarget>().iter(world) {
            let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("voxel_draw_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: view_target.main_texture_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_render_pipeline(render_pipeline);
            pass.set_vertex_buffer(0, gpu_vertex_buffer.buffer.slice(..));

            // IMPORTANT: vertex count must match what compute wrote
            let vertex_count = (CHUNK_COUNT * CHUNK_VOXELS_COUNT * 3) as u32;
            pass.draw(0..vertex_count, 0..1);

            return Ok(());
        }

        Ok(())
    }
}
