#import bevy_pbr::{
  mesh_view_bindings::{view, globals},
  mesh_bindings::mesh,
}
#import bevy_render::maths::affine3_to_square

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let mesh_uniform = mesh[vertex.instance_index];
    let world_from_local = affine3_to_square(mesh_uniform.world_from_local);
    out.world_position = world_from_local * vec4(vertex.position, 1.0);
    out.clip_position = view.clip_from_world * out.world_position;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // output the color directly
    return vec4(1.0, 1.0, 0.0, 1.0);
}