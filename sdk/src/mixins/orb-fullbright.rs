// Full block light and no directional shading, including non-HDR output.
fn hook(mut state: ItemRenderState, stack: &ItemStack) -> ItemRenderState {
    if stack.item.starts_with("lunatic:lampad_") || crate::game::lunatic_projectile_motion::ITEMS.iter().any(|id| *id == stack.item.as_ref()) || stack.item.starts_with("lunatic:yuuka_danmaku_") {
        state.hdr_light=15;
        for layer in &mut state.layers { layer.uses_block_light=false; }
    }
    state
}
