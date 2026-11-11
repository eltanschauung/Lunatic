fn draw(game: &crate::game::Game<'_>, alpha: f64, lines: &mut Vec<gizmos::Line>) {
    game.gourd_lines(alpha, lines);
}
