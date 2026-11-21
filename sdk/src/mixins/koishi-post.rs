fn post(game: &mut crate::game::Game<'_>) {
    use crate::game::koishi::GameLunaticKoishiExt as _;
    game.koishi_post();
}
