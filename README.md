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

Miko Stick, Lampad Torch, and Hauchiwa Fan remain inert handheld items. The
Lampad Torch does not place a block or emit light.

## Yin Yang Orbs and Purification Rod

| Item | ID | Name color | Stack size |
| --- | --- | --- | --- |
| Yin Yang Orb - Precision | `lunatic:yin_yang_orb_precision` | Crimson (`#dc143c`) | 16 |
| Yin Yang Orb - Homing | `lunatic:yin_yang_orb_homing` | Purple (`#800080`) | 16 |
| Yin Yang Orb - Bouncing | `lunatic:yin_yang_orb_bouncing` | Lime (`#00ff00`) | 16 |

All three are in the Lunatic creative tab and search. Inventory icons retain the
original 32×32 textures. Held, dropped, and flying orbs use a generated 32³ voxel
sphere, with longitude/latitude (equirectangular) UVs repeating the original orb
texture twice: one complete yin-yang face on each front/back hemisphere.
A white inner shell fills the source sprite's transparent padding without editing
its pixels. `python3 tools/generate_orbs.py` reproduces the model.

Fired and held orbs use a six-block light radius and 0.25 intensity, tinted to
their inventory hues with saturation reduced by 50% (HSV value preserved):
soft crimson (`#dc788c`), purple (`#804080`), and lime (`#80ff80`).
Lighting follows the interpolated projectile
on each client, using existing entity updates with no extra network messages.
The nearest eight loaded sources within 32 blocks are submitted to Horizon's
native visual lighting. These lights affect terrain and entity appearance, not
mob spawning or saved block light. Held lights support either hand and other
players/mobs. Sphere models have maximum light emission (15), so they stay lit
in darkness. Particle burst origins cast the same color at 0.125 intensity until
their particles expire; the drifting sprites share one light at their origin.

Orb explosions use Luanti's eight original particle sprites: 24 particles over
0.1 seconds on detonation and eight smaller particles over 0.08 seconds on bounce.
They live for one second, drift without gravity/collision, and retain glow level 8.
Precision uses the original red sprites; Homing and Bouncing use Luanti's
purple/lime colorize blend. Velocities and sizes are converted to Minecraft ticks
and blocks. The server batches burst events for clients within 32 blocks in the
same dimension; clients create all particles locally, with no per-particle packets.

Right-click the Purification Rod to cycle through ammo types currently in your
main inventory. Chat says `Selected ammo: Precision`, `Selected ammo: Homing`,
or `Selected ammo: Bouncing`. With no ammo it says `No Yin-Yang Orbs available.`
Left-click fires three orbs, 0.15 seconds apart, for one ammo item; the cooldown is
0.8 seconds. Creative still requires an orb, but keeps it. A depleted selection
falls back to the original Precision → Homing → Bouncing priority. With no ammo,
chat says `No Yin-Yang orbs available.` and the rod keeps its ordinary stick swing.

Flight and hits are authoritative on the server. Precision flies straight, Homing
tracks a visible target, and Bouncing uses gravity and damped reflections. The
native behavior uses Horizon's ordinary projectile entities, collision geometry,
damage, PvP rules, and entity networking. There is one compact fire request per
click; idle players send no weapon messages. Aiming while firing preserves movement
and sprint momentum: shot direction never changes the server player's pose.

### Sounds

The original Ogg files are kept byte-for-byte in Minecraft-style asset directories:

- `assets/lunatic/sounds/item/purification_rod/`: activation and failed homing lock.
- `assets/lunatic/sounds/entity/yin_yang_orb/`: each shot, bounce, impact, and close-range detonation.

`assets/lunatic/sounds.json` registers the events and their original hearing ranges.
The server triggers spatial sounds at the matching action, preserving the Luanti
volumes and volley spacing; nearby clients resolve the cached sound assets locally.
Ammo cycling and an empty rod add no invented sound. Subtitles have translations.

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
content packager/validator. The outputs are `target/horizon-mod/lunatic.hmod` (content) and
`target/horizon-mod/lunatic.hnmod` (native rod/orb behavior).
Copy `lunatic.hmod` into the instance's `horizon-mods/` directory, turn **Lunatic** on in
**Options → Horizon Settings → Mods**, and restart the game. A native game
rebuild is not required for texture-only changes. Install `lunatic.hnmod` in the
instance's `horizon-native-mods/` directory and use launcher **Build** to enable
weapon behavior. Clients and servers need both matching bundles. Rebuild and
install the native game when the Rust behavior changes.

Texture source: `luanti/lunatic_textures/mods/mcl_lun_items/textures/`
(`gohei.png`, `mcl_lun_items_miko_stick.png`,
`mcl_lun_items_lampad_torch.png`, and `hauchiwa_fan.png`).
The textures are preserved without modification. This port retains the Luanti
project's [GPLv3 license](LICENSE), also included in the content archive.
