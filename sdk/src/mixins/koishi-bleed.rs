fn bleed(game:&crate::game::Game<'_>,effect:mod_api::EffectKey,id:Option<i32>,ticks:i32) {
    use crate::game::koishi::GameLunaticKoishiExt as _;
 if game.on("lunatic") && game.effect_key("lunatic:bleeding")==Some(effect) && ticks%5==0 {game.koishi_bleed_tick(game.dim.id(),id,ticks);}
}
