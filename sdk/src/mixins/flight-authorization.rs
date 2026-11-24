pub fn apply(hub: &mut horizon_client_lib::lan::hub::Hub) {
    // The Hub derives its floating flag from game mode, missing temporary
    // may_fly granted by an active bat form. Clear it before the kick check.
    for index in 0..hub.guests.len() {
        let guest = &hub.guests[index];
        if guest.client_is_floating && guest.abilities.may_fly {
            hub.guests.update(index, |guest| guest.client_is_floating = false);
        }
    }
}
