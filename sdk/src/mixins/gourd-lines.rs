fn draw(game: &crate::game::Game<'_>, alpha: f64, lines: &mut Vec<gizmos::Line>) {
    use crate::game::lunatic_gourd::GameLunaticGourdExt as _;
    game.gourd_lines(alpha, lines);
}
