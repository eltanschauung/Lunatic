fn opacity(value:f32,game:&crate::game::Game<'_>,m:&Mob,camera:glam::DVec3)->f32 {value*game.koishi_opacity(m,camera)}
