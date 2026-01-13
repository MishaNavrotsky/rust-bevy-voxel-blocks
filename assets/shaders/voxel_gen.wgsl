struct Globals {
  chunk_size: u32,
};

struct ChunkCoord {
  coord: vec3<i32>,
  global_index: u32,
};

struct Vertex {
  pos: vec3<f32>,
  _pad: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> chunks: array<ChunkCoord>;
@group(0) @binding(2) var<storage, read_write> voxel_buffer: array<f32>;
@group(0) @binding(3) var<storage, read_write> vertecies_output: array<Vertex>;

fn height_at(x: u32, y: u32, chunk: u32, N: u32) -> f32 {
  let voxel_id =
      chunk * N * N * N +
      0u * N * N + // Z slice = 0 heightmap
      y * N +
      x;

  return voxel_buffer[voxel_id];
}

@compute @workgroup_size(8,8,1)
fn generate_vertecies(@builtin(global_invocation_id) gid: vec3<u32>) {
  let N = globals.chunk_size;
  let quads_per_row = N - 1u;
  let verts_per_quad = 6u;
  let quads_per_chunk = quads_per_row * quads_per_row;
  let verts_per_chunk = quads_per_chunk * verts_per_quad;

  let x = gid.x;
  let y = gid.y;
  let chunk = gid.z;

  // Bounds: only quads
  if (x >= N - 1u || y >= N - 1u) {
      return;
  }

  if (chunk >= arrayLength(&chunks)) {
      return;
  }

  let quad_index =
      chunk * quads_per_row * quads_per_row +
      y * quads_per_row +
      x;

  let vertex_base = quad_index * 6u;

  // Heights
  let h00 = height_at(x,     y,     chunk, N);
  let h10 = height_at(x + 1u,y,     chunk, N);
  let h01 = height_at(x,     y + 1u,chunk, N);
  let h11 = height_at(x + 1u,y + 1u,chunk, N);

  if (max(h00, max(h10, max(h01, h11))) < 0.001) {
    let p = vec3<f32>(0.0);
    for (var i = 0u; i < 6u; i++) {
        vertecies_output[vertex_base + i].pos = p;
    }
    return;
  }

  // World offset
  let chunk_offset =
      vec3<f32>(chunks[chunk].coord) * f32(N);

  let p00 = chunk_offset + vec3<f32>(f32(x),     h00, f32(y));
  let p10 = chunk_offset + vec3<f32>(f32(x + 1u),h10, f32(y));
  let p01 = chunk_offset + vec3<f32>(f32(x),     h01, f32(y + 1u));
  let p11 = chunk_offset + vec3<f32>(f32(x + 1u),h11, f32(y + 1u));

  // Triangle 1
  vertecies_output[vertex_base + 0u].pos = p00;
  vertecies_output[vertex_base + 1u].pos = p10;
  vertecies_output[vertex_base + 2u].pos = p01;

  // Triangle 2
  vertecies_output[vertex_base + 3u].pos = p10;
  vertecies_output[vertex_base + 4u].pos = p11;
  vertecies_output[vertex_base + 5u].pos = p01;
}

@compute @workgroup_size(4,4,4)
fn generate_height_map(@builtin(global_invocation_id) gid: vec3<u32>) {
  let chunk_index = gid.z / globals.chunk_size;
  let voxel_z     = gid.z % globals.chunk_size;

  if (chunk_index >= arrayLength(&chunks)) {
      return;
  }

  let voxel_local = vec3<u32>(gid.x, gid.y, voxel_z);

  let voxels_per_chunk =
      globals.chunk_size * globals.chunk_size * globals.chunk_size;

  let voxel_id =
      chunk_index * voxels_per_chunk +
      voxel_local.z * globals.chunk_size * globals.chunk_size +
      voxel_local.y * globals.chunk_size +
      voxel_local.x;

  if (chunks[chunk_index].global_index == 0xFFFFFFFFu) {
      voxel_buffer[voxel_id] = 0.0;
      return;
  }

  let world_pos =
      vec3<f32>(chunks[chunk_index].coord) * f32(globals.chunk_size) +
      vec3<f32>(voxel_local);

  voxel_buffer[voxel_id] =
      sin(world_pos.x) + cos(world_pos.y) + sin(world_pos.z);
}