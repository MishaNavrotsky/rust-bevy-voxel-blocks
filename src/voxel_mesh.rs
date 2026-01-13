use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3A;
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;

const CUBE_VERTS: [[f32; 3]; 24] = [
    // Front (+Z)
    [-0.5, -0.5, 0.5], // 0
    [0.5, -0.5, 0.5],  // 1
    [0.5, 0.5, 0.5],   // 2
    [-0.5, 0.5, 0.5],  // 3
    // Back (-Z)
    [0.5, -0.5, -0.5],  // 4
    [-0.5, -0.5, -0.5], // 5
    [-0.5, 0.5, -0.5],  // 6
    [0.5, 0.5, -0.5],   // 7
    // Left (-X)
    [-0.5, -0.5, -0.5], // 8
    [-0.5, -0.5, 0.5],  // 9
    [-0.5, 0.5, 0.5],   // 10
    [-0.5, 0.5, -0.5],  // 11
    // Right (+X)
    [0.5, -0.5, 0.5],  // 12
    [0.5, -0.5, -0.5], // 13
    [0.5, 0.5, -0.5],  // 14
    [0.5, 0.5, 0.5],   // 15
    // Top (+Y)
    [-0.5, 0.5, 0.5],  // 16
    [0.5, 0.5, 0.5],   // 17
    [0.5, 0.5, -0.5],  // 18
    [-0.5, 0.5, -0.5], // 19
    // Bottom (-Y)
    [-0.5, -0.5, -0.5], // 20
    [0.5, -0.5, -0.5],  // 21
    [0.5, -0.5, 0.5],   // 22
    [-0.5, -0.5, 0.5],  // 23
];

const CUBE_INDICES: [u32; 36] = [
    // Front
    0, 1, 2, 0, 2, 3, // Back
    4, 5, 6, 4, 6, 7, // Left
    8, 9, 10, 8, 10, 11, // Right
    12, 13, 14, 12, 14, 15, // Top
    16, 17, 18, 16, 18, 19, // Bottom
    20, 21, 22, 20, 22, 23,
];

fn cube() -> VertexAttributeValues {
    VertexAttributeValues::Float32x3(CUBE_VERTS.to_vec())
}

fn cube_indices() -> Indices {
    Indices::U32(CUBE_INDICES.to_vec())
}

pub fn make_test_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, cube());

    mesh.insert_indices(cube_indices());

    mesh
}
