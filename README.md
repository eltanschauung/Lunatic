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

Miko Stick and Hauchiwa Fan remain inert handheld items. The single Lampad Torch
now has its Luanti attacks and held light; it does not place a block. See
[Lampad Torch](docs/lampad-torch.md) for controls, values and native adaptations.

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
or `Selected ammo: Bouncing`; only the ammo name uses its inventory color, as does the held-item HUD name. With no
ammo it says `No Yin-Yang Orbs available.`
Left-click fires three orbs, 0.15 seconds apart, for one ammo item; the cooldown is
0.8 seconds. The rod has 250 uses and spends one durability per successful volley.
Creative still requires an orb, but keeps it and spends no durability. A depleted selection
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

Lunatic supplies 41 appearances for Horizon's **Character Select** menu, in this order:

1. Hakurei Reimu
2. Kirisame Marisa
3. Kazami Yuuka
4. Alice Margatroid
5. Daiyousei
6. Cirno
7. Hong Meiling
8. Koakuma
9. Patchouli Knowledge
10. Izayoi Sakuya
11. Remilia Scarlet
12. Flandre Scarlet
13. Chen
14. Konpaku Youmu
15. Saigyouji Yuyuko
16. Yakumo Ran
17. Yakumo Yukari
18. Wriggle Nightbug
19. Inaba Tewi
20. Reisen Udongein Inaba
21. Houraisan Kaguya
22. Fujiwara no Mokou
23. Ibuki Suika
24. Shameimaru Aya
25. Kawashiro Nitori
26. Inubashiri Momiji
27. Kochiya Sanae
28. Nagae Iku
29. Hinanawi Tenshi
30. Komeiji Satori
31. Komeiji Koishi
32. Miyako Yoshika
33. Kaku Seiga
34. Ibaraki Kasen
35. Houjuu Nue
36. Minamitsu Murasa
37. Yorigami Shion
38. Yorigami Jo'on
39. Clownpiece
40. Junko
41. Kudamaki Tsukasa

Browse a character to see its idle 3D preview, then press **Select Character**.
Singleplayer and Multiplayer remain disabled until a choice is confirmed.
Your choice is saved for this instance and used for your player model. Steve
is only a fallback for an appearance that fails to load; he is not a choice.
Requires the Horizon fork's Character Select support.

Character definitions default to the slim player model and `character_chat: true`; either can be overridden explicitly. When `texture` is omitted, `lunatic:NAME` resolves to `lunatic:entity/character/NAME`:
with the fork's character-chat support, your ordinary chat is displayed locally as
`Selected Character Name : message`. The same name appears on your own visible
nametag. Optional `color` uses Minecraft text colors (for example `"red"` or
`"#ff5555"`) for the name in both places; Reimu is red. Account identity, signed messages, commands,
and private-message formatting are preserved. Matching Horizon servers validate character choices and synchronize their appearances,
names and colors to other clients.

The original 64×32 and 64×64 skins from `mcl_lun_races/textures/` are packaged unchanged;
Horizon converts their legacy layout in memory when loading them. The roster
is declared in `assets/lunatic/horizon/characters.json`.


### Races, size and starting items

`assets/lunatic/horizon/characters.json` is the source of character traits:

```json
{
  "race": "fairy,ice_fairy,youkai",
  "size": 0.7,
  "starting_items": [
    {"item": "minecraft:cookie", "count": 16},
    {"choices": ["minecraft:poppy", "minecraft:dandelion"],
     "min_count": 1, "max_count": 4, "rolls": 3}
  ]
}
```

`race` accepts comma-separated tags, with surrounding spaces ignored. `size`
defaults to `1.0` and is clamped to `0.7`–`1.2`. It uses Minecraft's scale
attribute for the model, collision and selection boxes, pose dimensions,
first-person eyes, targeting and projectile origins. Other attribute modifiers
continue to compose normally. Cirno, Daiyousei and Clownpiece are `0.7`.

Each `fairy` gets Luanti's one extra jump while airborne. Release jump after the
normal jump and press it again to boost. Slow falling follows the boost; another
press cancels it. Landing resets the ability. Water, riding, sleep, death, creative
flight and elytra flight disable it. Float uses Horizon's vanilla Slow Falling
status and gravity/fall-damage pipeline; cancelling it preserves a potion's effect.
The original jump and cancellation sounds live in `sounds/entity/fairy/`.
Normal player input also drives the server and movement replay, without a second
input channel or per-tick ability packets.

Kits use existing Minecraft/Lunatic items, preserving Luanti's counts and random
flower/ammo draws. Unported items are omitted. `starting_items` defaults to an
empty list; entries support `item`, `count`, `choices`, `min_count`, `max_count`
and `rolls` (one by default). The server grants each character's kit once per
player per world. Grants and pending items are saved with inventory in player
NBT. Reconnecting, dying or switching back never duplicates a kit. A full inventory
keeps its rolled remainder pending until room is available. Unknown items are
skipped.

Client and server need matching content catalogs and native builds. To update
race, size or kit definitions, rebuild and install the content package on both;
Rust changes require a native rebuild.

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

## Player death sound

Lunatic redirects the standard player-death event to `pldead00.ogg` from the Luanti game, stored at `assets/lunatic/sounds/entity/player/pldead00.ogg`. The normal positional player sound pipeline and Players volume category are retained, with sound volume `0.5` and fixed original pitch `1.0` (no Minecraft pitch randomization). Nearby players hear the death in its dimension, while the dying player hears the normal local death event once. Missing Lunatic sound resources preserve Minecraft's original sound.

## Crow tengu flight and Hauchiwa Fan

Characters whose comma-separated `race` includes `crow_tengu` can glide through Minecraft’s normal elytra controls,
including with an empty chest slot or chest armor. Their virtual glider does not consume equipment durability. Level, unboosted flight settles at 18 blocks/s with a 5:1 glide ratio; pitch steering and firework boosts still use the normal flight pipeline.
The validated character catalog supplies the trait on both the client and world host. Crow tengu cannot start or maintain a glide above 11 armor points, even with an elytra equipped; full gold armor (11 points) remains eligible. The limit uses the existing computed armor attribute cache and normal movement/equipment updates, without inventory scans or extra networking.

While gliding, a crow tengu can right-click `lunatic:hauchiwa_fan` for the normal duration-1 firework boost.
The fan has 250 durability, costs 5 durability per successful boost, has no explosive payload, and has a ten-second (200-tick) cooldown shared by both hands and all copies. Cooldown-blocked clicks cost nothing; creative mode and Unbreaking use normal durability rules.
The standard item cooldown packet and HUD display the cooldown. Non-crow characters keep their normal equipment-based gliding.
No additional mod networking is used for the boost.

Native behavior 0.7.8 and content package 0.7.9 include the fan durability definition. No character JSON schema changes are required.
The isolated multiplayer check is `tools/test_crow_tengu_e2e.py` (`--game`, `--ctl`, `--resources`, `--mod`, `--output`).
It creates temporary server/client homes and checks armored/empty-chest gliding, level-flight speed and ratio, both-hand durability and cooldown behavior, fan breakage at 250 damage, and normal elytra behavior after switching to a non-crow character.


## Ibuki Gourd and Oni (鬼)

Ibuki Suika starts with one light-purple Ibuki Gourd (`lunatic:ibuki_gourd`), also available in the Lunatic creative tab. Hold use for the normal 1.6-second potion-style drink. Completing it keeps the gourd, grants Oni (`lunatic:oni`) for a uniformly random 12–20 seconds and starts a four-minute cooldown shared by both hands and all copies. Interrupted drinks grant no effect or cooldown. The effect icon uses the gourd texture. On natural expiry, the server emits the normal eating-completion burp and removes a random 2–5 food points (one to two-and-a-half hunger icons), clamped at zero; early removal and death do not trigger this penalty. The same expiry grants Drunk (`lunatic:drunk`) for a random 6–10 seconds: Darkness I plus Nausea I through the normal fog, lighting and distortion pipelines. Its custom icon is loaded from `assets/lunatic/textures/mob_effect/drunk.png` through the standard effect atlas. Drunk remains a single synchronized/saved effect and preserves stronger concurrent vanilla effects.

Oni supplies Strength II (+6 damage) to empty-hand punches and Strength I (+3 damage) to held-item melee attacks, with at least Knockback II. Higher knockback enchantments remain effective, normal sprint and attack-strength rules apply, and projectiles are unchanged. Native potion synchronization, removal, persistence, drinking sounds and cooldown packets handle the effect without custom networking. Existing characters whose starting kit was already granted keep their saved inventory; the kit entry applies when Suika's starting kit is first awarded.

`tools/test_gourd_e2e.py` runs the real drinking, cooldown and melee comparisons in temporary multiplayer homes.


## Yuuka's Parasol

`lunatic:yuuka_parasol` uses the supplied Luanti texture, a green inventory/HUD name and the standard handheld item model. It is available in the Lunatic creative tab and Kazami Yuuka's starting kit, with iron-sword damage, attack speed, durability and tool behavior, and intrinsic Knockback III on every copy, including creative-menu and starting-kit stacks. Crafting/enchantment changes retain Knockback III; grindstones cannot remove or redeem it.

Characters with the `yuuka` race permanently receive Strength I and a minimum of Knockback I on melee attacks. Oni/Yuuka/vanilla Strength use the strongest applicable Strength bonus, and stronger knockback enchantments are preserved. Successful empty-hand mob hits by Yuuka or an Oni-affected player play the shared original `entity/explosion_old.ogg` at the mob origin at half volume. Failed or invulnerable hits and projectiles do not play it. Homing-orb detonations reference the renamed file with their existing volume unchanged.

Applying Oni plays Minecraft's Ender Dragon ambient sound at the affected entity's position at 50% volume and normal pitch. The built-in event selects among its four `growl1–4` clips; no Minecraft audio is bundled. Drinks and command applications share the same server-side application hook, without tick or client-synchronization echoes.

While Oni is active, the camera uses half of Speed I's FOV widening (+5% at normal walking speed with FOV Effects at 100%), through the standard camera modifier before zoom and smoothing. This is visual only: movement speed is unchanged, the FOV Effects slider still scales it, and first-person spyglass zoom stays intact.

Successful Oni/Yuuka explosion punches also emit one vanilla explosion puff at the mob's body center, using normal nearby particle delivery. This is visual only; damage, knockback and blocks are unchanged. Missed, invulnerable and held-weapon hits emit neither the punch sound nor the puff.

Right-click Yuuka's Parasol to toggle fairy-style slow falling for any character. Opening plays a random vanilla Ender Dragon wing/flap clip at half volume. Another click or a fresh Space press cancels it with the fairy cancel sound; holding use never repeatedly toggles. It stays armed on the ground and when switching items, uses the same Slow Falling motion flag as fairies/potions, and cancelling does not remove an actual Slow Falling potion. State updates go only to the owner on initialization or when the toggle changes; jump cancellation uses ordinary player input packets.

Default full, half and container hearts, including blinking variants, use Lunatic’s 17×17 textures in `assets/lunatic/textures/gui/sprites/hud/heart/`. Their complete images are scaled into the usual 9×9 HUD space, keeping heart spacing and bar dimensions intact.

Heart sprite overrides are discovered once when the atlas is prepared. Adding or editing matching PNGs in that folder only requires rebuilding the content package.

`lunatic:psilocybe_cubensis` is a decorative mushroom block named **Psilocybe Cubensis** in pink, available in the Lunatic creative tab. Placed blocks deterministically choose equally between adult, cluster and juvenile OBJ models, each with four equally likely quarter-turn orientations, following MushroomsPlus’s blockstate-variant pattern. The inventory uses the supplied mushroom icon. MTL files bind `#cap` and `#stem` to `cubensis_cap` and `cubensis_stem`; the block adds no edible, potion, growth or weapon functionality.
