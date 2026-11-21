fn opacity(value:f32,game:&crate::game::Game<'_>,m:&Mob,camera:glam::DVec3)->f32 {
    use crate::game::koishi::GameLunaticKoishiExt as _;
value*game.koishi_opacity(m,camera)}
