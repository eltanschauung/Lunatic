fn apply(game: &mut crate::game::Game<'_>) {
    use crate::game::commands::Target;
    use horizon_client_lib::lan::waypoints::team_rgb;

    let Some(host) = game.lan.as_ref() else { return };
    for guest in host.hub.guests.iter().filter(|guest| guest.placed) {
        let Some(color) = game.character_for(guest.uuid).and_then(|character| character.color.as_deref()) else {
            continue;
        };
        let rgb = if let Some(hex) = color.strip_prefix('#') {
            i32::from_str_radix(hex, 16).ok().filter(|value| (0..=0xff_ffff).contains(value))
        } else {
            team_rgb(color)
        };
        if let Some(rgb) = rgb {
            game.cmds.waypoint_icons.entry(Target::Guest(guest.conn)).or_default().color = Some(rgb);
        }
    }
}
