// SPDX-License-Identifier: GPL-3.0-or-later
fn drunk_effect(
    effects: &PlayerEffects,
    id: impl AsRef<str>,
) -> core::ops::ControlFlow<Option<&EffectInstance>> {
    let id = id.as_ref();
    if !matches!(id, "darkness" | "nausea") {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(drunk) = effects
        .list
        .iter()
        .find(|effect| effect.id == "lunatic:drunk")
    else {
        return core::ops::ControlFlow::Continue(());
    };
    let active = effects.list.iter().find(|effect| effect.id == id);
    let priority = |effect: &EffectInstance| {
        (
            effect.amplifier,
            if effect.duration == -1 {
                i32::MAX
            } else {
                effect.duration
            },
        )
    };
    let effective = match active {
        Some(active) if priority(active) >= priority(drunk) => active,
        _ => drunk,
    };
    // The standard darkness/fog and nausea/distortion readers share get().
    // Keep only the actual Drunk instance for ticking, saving, HUD and packets.
    core::ops::ControlFlow::Break(Some(effective))
}
