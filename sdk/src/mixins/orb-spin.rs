// SPDX-License-Identifier: GPL-3.0-or-later
// Spin the existing mesh each rendered frame. No model variants or item metadata.
fn hook(
    game: &crate::game::Game<'_>,
    p: &horizon_client::projectiles::Projectile,
    partial: f64,
    pose: &mut glam::Mat4,
) {
    if crate::game::lunatic_gourd_motion::ours(p) {
        let (_, _, position) = pose.to_scale_rotation_translation();
        let axis = ::mod_lunatic::gourd_physics::forward(p.yaw, p.pitch);
        // The model's +Y is its nozzle: point it back toward the thrower.
        let nozzle = -glam::Vec3::new(axis[0] as f32, axis[1] as f32, axis[2] as f32);
        let rotation = glam::Quat::from_rotation_arc(glam::Vec3::Y, nozzle);
        *pose = glam::Mat4::from_rotation_translation(rotation, position);
        return;
    }
    if let Some(index) = crate::game::lunatic_yuuka_flight::FLOWERS
        .iter()
        .position(|id| *id == p.kind.id())
    {
        const SPRITES: [&str; 3] = [
            "lunatic:item/yuuka_danmaku_flower_large",
            "lunatic:item/yuuka_danmaku_flower_medium",
            "lunatic:item/yuuka_danmaku_flower_small",
        ];
        let sprites = &game.gui.atlas.sprites;
        if let (Some(image), Some(reference)) =
            (sprites.get(SPRITES[index]), sprites.get(SPRITES[0]))
        {
            let unit = reference.w.max(1) as f32;
            let scale = glam::Vec3::new(image.w as f32 / unit, image.h as f32 / unit, 1.0);
            let center = glam::Vec3::new(0.0, 2.0 / 16.0, 0.0);
            *pose *= glam::Mat4::from_translation(center)
                * glam::Mat4::from_scale(scale)
                * glam::Mat4::from_translation(-center);
        }
        return;
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
    if crate::game::lunatic_projectile_motion::variant(p).is_none() {
        return;
    }
    // Camera-local -Z roll is clockwise on screen. 40 game ticks = two seconds.
    let angle = -((p.age % 40) as f32 + partial as f32) * std::f32::consts::TAU / 40.0;
    let rotation = glam::Quat::from_rotation_z(angle);
    // Ground display lifts the model by two texels: keep that centre fixed.
    let centre = glam::Vec3::new(0.0, 2.0 / 16.0, 0.0);
    *pose *= glam::Mat4::from_rotation_translation(rotation, centre - rotation * centre);
}
