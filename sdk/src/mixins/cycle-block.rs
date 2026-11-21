// SPDX-License-Identifier: GPL-3.0-or-later
fn cycle_block(
    game: &mut crate::game::Game<'_>,
    stack: Option<&ItemStack>,
    phase: mod_api::UsePhase,
) -> std::ops::ControlFlow<Option<mod_api::InteractionResult>> {
    use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    if phase != mod_api::UsePhase::Block {
        return std::ops::ControlFlow::Continue(());
    }
    if let Some(stack) = stack {
        let mut claimed = game.lunatic_fan_boost(stack);
        if !claimed {
            game.lunatic_cycle(stack, &mut claimed);
        }
        if claimed {
            return std::ops::ControlFlow::Break(Some(mod_api::InteractionResult::Consume));
        }
    }
    std::ops::ControlFlow::Continue(())
}
