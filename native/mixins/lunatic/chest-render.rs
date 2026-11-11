fn chest(
    m: &crate::mobs::Mob,
    alpha: f32,
    base: glam::Mat4,
    atlas: crate::mobs::render::Atlas<'_>,
    light: u32,
    data: &crate::mobs::MobData,
    visible: bool,
    out: &mut Vec<crate::mesher::Vertex>,
) {
    crate::mobs::render::lunatic_chest::render_mob(
        m, alpha, base, atlas, light, data, visible, out,
    );
}
