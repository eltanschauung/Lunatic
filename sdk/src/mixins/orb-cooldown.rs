fn hook(p: &horizon_client::projectiles::Projectile, bypass: &mut bool) {
    if crate::game::lunatic_weapons::projectile_weapon(p)
        .is_some_and(|weapon| weapon.ignores_hurt_cooldown)
    {
        *bypass = true;
    }
}
