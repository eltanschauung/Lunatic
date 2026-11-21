// SPDX-License-Identifier: GPL-3.0-or-later
fn gourd_use_start(game: &mut crate::game::Game<'_>, stack: &ItemStack, entity: Option<i32>, canceled: &mut bool) {
    use crate::game::lunatic_gourd::GameLunaticGourdExt as _;
    if entity.is_none() && game.on("lunatic")
        && stack.item.as_ref() == "lunatic:ibuki_gourd" && game.gourd_use_unavailable() {
        *canceled = true;
    }
}
