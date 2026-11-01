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

Lunatic supplies 29 appearances for Horizon's **Character Select** menu, in this order:

1. Hakurei Reimu
2. Kirisame Marisa
3. Kazami Yuuka
4. Alice Margatroid
5. Daiyousei
6. Cirno
7. Hong Meiling
8. Patchouli Knowledge
9. Izayoi Sakuya
10. Remilia Scarlet
11. Flandre Scarlet
12. Konpaku Youmu
13. Saigyouji Yuyuko
14. Wriggle Nightbug
15. Inaba Tewi
16. Reisen Udongein Inaba
17. Houraisan Kaguya
18. Fujiwara no Mokou
19. Ibuki Suika
20. Shameimaru Aya
21. Kawashiro Nitori
22. Inubashiri Momiji
23. Kochiya Sanae
24. Nagae Iku
25. Hinanawi Tenshi
26. Komeiji Satori
27. Komeiji Koishi
28. Houjuu Nue
29. Minamitsu Murasa

Browse a character to see its idle 3D preview, then press **Select Character**.
Singleplayer and Multiplayer remain disabled until a choice is confirmed.
Your choice is saved for this instance and used for your player model. Steve
is only a fallback for an appearance that fails to load; he is not a choice.
Requires the Horizon fork's Character Select support.

All 29 characters use the slim player model. Each sets `character_chat: true`:
with the fork's character-chat support, your ordinary chat is displayed locally as
`Selected Character Name : message`. The same name appears on your own visible
nametag. Optional `color` uses Minecraft text colors (for example `"red"` or
`"#ff5555"`) for the name in both places; Reimu is red. Account identity, signed messages, commands,
and private-message formatting are preserved. Remote players' character choices
are not yet synchronized; other clients keep their normal chat presentation.

The original 64×32 and 64×64 skins from `mcl_lun_races/textures/` are packaged unchanged;
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
