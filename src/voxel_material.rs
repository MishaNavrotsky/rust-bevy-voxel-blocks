use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct VoxelMaterial {
    #[uniform(0)]
    pub color: Vec4,
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/voxel.wgsl".into()
    }
}
