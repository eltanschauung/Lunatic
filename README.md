# Lunatic

Lunatic content for Horizon. Future content ported from the Luanti game belongs
in this mod.

The initial item is **Purification Rod** (`lunatic:purification_rod`). It appears
in the **Lunatic** creative tab and creative search, with a red hover name. Its
original 32×32 `gohei.png` texture uses Minecraft's handheld model, including
extruded pixel edges and sword-style holding transforms.

The rod is an inert item: it has no special attacks, spells, projectiles,
crafting recipe, durability, or use action.

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

Texture source: `luanti/lunatic_textures/mods/mcl_lun_items/textures/gohei.png`.
The texture is preserved without modification. This port retains the Luanti
project's [GPLv3 license](LICENSE), also included in the content archive.
