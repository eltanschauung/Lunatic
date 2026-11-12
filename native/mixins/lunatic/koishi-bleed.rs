fn bleed(game:&crate::game::Game<'_>,effect:&str,id:Option<i32>,ticks:i32) {
 if game.native_on("lunatic") && effect=="lunatic:bleeding" && ticks%5==0 {game.koishi_bleed_tick(game.dim.id(),id,ticks);}
}
