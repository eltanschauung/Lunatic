// SPDX-License-Identifier: GPL-3.0-or-later
// Redirect the existing player-death event, keeping its position, source, gain and network path.
fn player_death_sound(audio: &mut Audio, event: &str) {
    if event != "minecraft:entity.player.death" && event != "entity.player.death" {
        return;
    }
    let registry = &mut audio.engine.registry;
    const REPLACEMENT: &str = "lunatic:entity.player.death";
    if registry.weight(REPLACEMENT) == 0 {
        return; // Missing/disabled content keeps Minecraft's normal sound.
    }
    if registry.get("minecraft:entity.player.death").is_some_and(|entry| {
        entry.sounds.len() == 1 && entry.sounds[0].event && entry.sounds[0].location == REPLACEMENT
    }) {
        return; // One alias per resource load, not one new registration per death.
    }
    registry.add_file(
        "minecraft",
        br#"{"entity.player.death":{"replace":true,"subtitle":"subtitles.entity.player.death","sounds":[{"name":"lunatic:entity.player.death","type":"event"}]}}"#,
        &|_, _| true,
    );
}
