// SPDX-License-Identifier: GPL-3.0-or-later
fn cycle(game: &mut crate::game::Game<'_>, stack: &ItemStack) -> std::ops::ControlFlow<Option<bool>> {
    use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    let mut claimed = game.lunatic_fan_boost(stack);
    if !claimed {
        game.lunatic_cycle(stack, &mut claimed);
    }
    if claimed {
        std::ops::ControlFlow::Break(Some(true))
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
