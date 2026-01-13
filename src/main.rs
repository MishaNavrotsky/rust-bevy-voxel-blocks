use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

mod chunks_partition;
mod fly_camera;
mod voxel_compute_grid;
mod voxel_material;
mod voxel_mesh;
mod voxel_render;

use fly_camera::FlyCamera;
use fly_camera::fly_camera;
use voxel_material::VoxelMaterial;
use voxel_mesh::make_test_mesh;

use crate::chunks_partition::{VisibleChunks, chunks_partition};
use crate::voxel_compute_grid::VoxelComputeGridPlugin;

fn grab_cursor(mut q: Query<&mut CursorOptions>) {
    let mut cursor = q.single_mut().unwrap();
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::Immediate,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont::default(),
                text_color: Color::WHITE,
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: false,
                    min_fps: 30.0,
                    target_fps: 240.0,
                },
            },
        })
        .add_plugins(VoxelComputeGridPlugin)
        .add_plugins(MaterialPlugin::<VoxelMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, fly_camera)
        .init_resource::<VisibleChunks>()
        .add_systems(Update, chunks_partition)
        .add_systems(Startup, grab_cursor)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let mesh = meshes.add(make_test_mesh());
    let material = materials.add(VoxelMaterial { color: Vec4::ONE });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d::<VoxelMaterial>(material),
        Transform::from_scale(Vec3::new(10.0, 10.0, 10.0)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0),
        FlyCamera {
            speed: 10.0,
            sensitivity: 0.001,
            pitch: 0.0,
            yaw: 0.0,
        },
    ));
}
