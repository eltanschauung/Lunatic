// SPDX-License-Identifier: GPL-3.0-or-later
// Spin the existing mesh each rendered frame. No model variants or item metadata.
fn hook(
    p: &horizon_client::projectiles::Projectile,
    partial: f64,
    pose: &mut glam::Mat4,
) {
    if horizon_client::projectiles::gourd_motion::ours(p) {
        let (scale,_,position)=pose.to_scale_rotation_translation();
        let speed=p.velocity.distance_squared(horizon_core::geometry::Vec3::ZERO).sqrt() as f32;
        let angle=if speed>0.02{(p.age as f32+partial as f32)*0.45}else{0.0};
        *pose=glam::Mat4::from_scale_rotation_translation(scale*1.3,glam::Quat::from_rotation_x(angle),position);return;
    }
    if p.kind.id().starts_with("lunatic:lampad_star_") {
        // The star's broad faces are in XY. Replace the camera billboard with
        // a world-space XZ pose, retaining the renderer's position and scale.
        let (scale, _, position) = pose.to_scale_rotation_translation();
        *pose = glam::Mat4::from_scale_rotation_translation(
            scale,
            glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            position,
        );
        return;
    }
    if horizon_client::projectiles::lunatic_motion::variant(p).is_none() { return; }
    // Camera-local -Z roll is clockwise on screen. 40 game ticks = two seconds.
    let angle = -((p.age % 40) as f32 + partial as f32) * std::f32::consts::TAU / 40.0;
    let rotation = glam::Quat::from_rotation_z(angle);
    // Ground display lifts the model by two texels: keep that centre fixed.
    let centre = glam::Vec3::new(0.0, 2.0 / 16.0, 0.0);
    *pose *= glam::Mat4::from_rotation_translation(rotation, centre - rotation * centre);
}
