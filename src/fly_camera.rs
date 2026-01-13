use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
pub struct FlyCamera {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

pub fn fly_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        mouse_delta += ev.delta;
    }

    for (mut transform, mut cam) in &mut query {
        cam.yaw -= mouse_delta.x * cam.sensitivity;
        cam.pitch -= mouse_delta.y * cam.sensitivity;

        cam.pitch = cam.pitch.clamp(-1.54, 1.54);

        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);

        let mut dir = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            dir.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            dir.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }
        if keys.pressed(KeyCode::Space) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::ControlLeft) {
            dir.y -= 1.0;
        }

        if dir != Vec3::ZERO {
            let mut speed_mult = 1.0;
            if keys.pressed(KeyCode::ShiftLeft) {
                speed_mult = 5.0;
            }

            let forward = transform.rotation * dir.normalize();
            transform.translation += forward * cam.speed * speed_mult * time.delta_secs();
        }
    }
}
