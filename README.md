# Lunatic

Lunatic content for Horizon. Future content ported from the Luanti game belongs
in this mod.

The items appear in the **Lunatic** creative tab and creative search:

| Item | ID | Hover-name color |
| --- | --- | --- |
| Purification Rod | `lunatic:purification_rod` | Red |
| Miko Stick | `lunatic:miko_stick` | Light green |
| Lampad Torch | `lunatic:lampad_torch` | Purple |
| Hauchiwa Fan | `lunatic:hauchiwa_fan` | Orange |

Their original 32×32 textures use Minecraft's handheld model, including
extruded pixel edges and sword-style holding transforms.

All four items are inert: they have no special attacks, spells, projectiles,
crafting recipes, durability, use actions, or fuel behavior. The Lampad Torch
is a handheld item and does not place a block or emit light.

## Characters

Lunatic supplies eleven appearances for Horizon's **Character Select** menu:
Reimu Hakurei, Marisa Kirisame, Reisen Udongein Inaba, Fujiwara no Mokou,
Kaguya Houraisan, Sanae Kochiya, Aya Shameimaru, Tewi Inaba, Cirno,
Daiyousei, and Wriggle Nightbug.

Browse a character to see its idle 3D preview, then press **Select Character**.
Singleplayer and Multiplayer remain disabled until a choice is confirmed.
Your choice is saved for this instance and used for your player model. Steve
is only a fallback for an appearance that fails to load; he is not a choice.
Requires the Horizon fork's Character Select support.

The original 64×32 skins from `mcl_lun_races/textures/` are packaged unchanged;
Horizon converts their legacy layout in memory when loading them. The roster
is declared in `assets/lunatic/horizon/characters.json`.

## Build and install

With a Horizon checkout beside this repository, run:

```sh
python3 tools/build.py
```

The script uses the fork's Rust setup when available and Horizon's official
content packager/validator. The output is `target/horizon-mod/lunatic.hmod`.
Copy it into the instance's `horizon-mods/` directory, turn **Lunatic** on in
**Options → Horizon Settings → Mods**, and restart the game. A native game
rebuild is not required.

Texture source: `luanti/lunatic_textures/mods/mcl_lun_items/textures/`
(`gohei.png`, `mcl_lun_items_miko_stick.png`,
`mcl_lun_items_lampad_torch.png`, and `hauchiwa_fan.png`).
The textures are preserved without modification. This port retains the Luanti
project's [GPLv3 license](LICENSE), also included in the content archive.
