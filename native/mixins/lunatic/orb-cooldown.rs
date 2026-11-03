fn hook(p: &horizon_client::projectiles::Projectile, bypass: &mut bool) {
    if horizon_client::projectiles::lunatic_motion::variant(p).is_some() { *bypass=true; }
}
