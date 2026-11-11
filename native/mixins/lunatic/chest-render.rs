fn chest(
    m: &crate::mobs::Mob,
    alpha: f32,
    base: glam::Mat4,
    atlas: crate::mobs::render::Atlas<'_>,
    light: u32,
    model: &crate::mobs::Part,
    poses: &crate::mobs::anim::Poses,
    skin: &str,
    visible: bool,
    out: &mut Vec<crate::mesher::Vertex>,
) {
    crate::mobs::render::lunatic_chest::render(
        m, alpha, base, atlas, light, model, poses, skin, visible, out,
    );
}
