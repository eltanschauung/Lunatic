fn hit(game:&mut crate::game::Game<'_>,index:usize) {if let Some(m)=game.mobs.get(index) {let id=m.id;game.koishi_knife_hit(id);}}
