// SPDX-License-Identifier: GPL-3.0-or-later
fn death_pitch(value: f32, audio: &Audio, event: &str) -> f32 {
    if matches!(event, "minecraft:entity.player.death" | "entity.player.death" | "lunatic:entity.player.death")
        && audio.engine.registry.weight("lunatic:entity.player.death") > 0 {
        1.0
    } else { value }
}
