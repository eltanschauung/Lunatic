// Full block light and no directional shading, including non-HDR output.
fn hook(mut state: ItemRenderState, stack: &ItemStack) -> ItemRenderState {
    if horizon_client::projectiles::lunatic_motion::ITEMS.iter().any(|id| *id == stack.item.as_ref()) {
        state.hdr_light=15;
        for layer in &mut state.layers { layer.uses_block_light=false; }
    }
    state
}
