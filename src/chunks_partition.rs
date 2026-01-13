use bevy::{
    camera::primitives::{Aabb, Frustum},
    math::Affine3A,
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::encase::private::Length},
};

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct VisibleChunks {
    pub chunks: Vec<(IVec3, u32)>,
}

pub const CHUNK_SIZE: f32 = 16.0;
pub const CHUNK_EXTENT_XZ: i32 = 6;
pub const CHUNK_EXTENT_Y: i32 = 4;
pub const CHUNKS_XZ: usize = (CHUNK_EXTENT_XZ * 2 + 1) as usize;
pub const CHUNKS_Y: usize = (CHUNK_EXTENT_Y * 2 + 1) as usize;
pub const CHUNK_COUNT: usize = CHUNKS_XZ * CHUNKS_XZ * CHUNKS_Y;
pub const CHUNK_VOXELS_COUNT: usize =
    CHUNK_SIZE as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize;

const HALF: Vec3A = Vec3A::splat(CHUNK_SIZE * 0.5);
const LOCAL_AABB: Aabb = Aabb {
    center: HALF,
    half_extents: HALF,
};

fn chunk_global_index(chunk: IVec3) -> u32 {
    let x_index = (chunk.x + CHUNK_EXTENT_XZ) as u32;
    let y_index = (chunk.y + CHUNK_EXTENT_Y) as u32;
    let z_index = (chunk.z + CHUNK_EXTENT_XZ) as u32;

    x_index + y_index * CHUNKS_XZ as u32 + z_index * CHUNKS_XZ as u32 * CHUNKS_XZ as u32
}

pub fn chunks_partition(
    query: Query<(&GlobalTransform, &Frustum), With<Camera3d>>,
    mut visible: ResMut<VisibleChunks>,
) {
    if visible.chunks.len() != CHUNK_COUNT {
        visible.chunks.resize(CHUNK_COUNT, (IVec3::ZERO, u32::MAX));
    } else {
        visible.chunks.fill((IVec3::ZERO, u32::MAX));
    }

    let Ok((transform, frustum)) = query.single() else {
        return;
    };

    let cam_pos = transform.translation();
    let cam_chunk = (cam_pos / CHUNK_SIZE).floor().as_ivec3();

    let mut idx = 0;
    for dx in -CHUNK_EXTENT_XZ..=CHUNK_EXTENT_XZ {
        for dy in -CHUNK_EXTENT_Y..=CHUNK_EXTENT_Y {
            for dz in -CHUNK_EXTENT_XZ..=CHUNK_EXTENT_XZ {
                let chunk_coord = cam_chunk + IVec3::new(dx, dy, dz);
                let world_from_local =
                    Affine3A::from_translation(chunk_coord.as_vec3() * CHUNK_SIZE);
                if frustum.intersects_obb(
                    &LOCAL_AABB,
                    &world_from_local,
                    true, // intersect near plane
                    true, // intersect far plane
                ) {
                    let chunk_coord = cam_chunk + IVec3::new(dx, dy, dz);

                    visible.chunks[idx] = (chunk_coord, chunk_global_index(chunk_coord));
                    idx += 1;
                }
            }
        }
    }
}
