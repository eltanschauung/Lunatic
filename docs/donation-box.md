# Donation Box

`lunatic:donation_box` is in the Lunatic creative tab. Its name is crimson and its lore is gray, matching the Luanti original. It has 45 persistent storage slots (five rows), supports hoppers and comparators, and spills its contents when broken through Horizon's standard mod-container pipeline. It uses wooden mining sounds, hardness 2.5 and blast resistance 2.5.

The original ten node boxes are converted from centered Luanti coordinates into Minecraft model coordinates, with the same directional collision boxes. The original front, side and wood textures are copied unchanged into `assets/lunatic/textures/block/`. Front/side source: `Mineclonia-Lunatic/mods/mcl_lun_nodes/textures/`; wood source: `lunatic_textures/games/lunatic/mods/ITEMS/mcl_core/textures/default_wood.png`.

Craft it with three sticks across the top, planks/ink sac/planks in the middle, and three planks across the bottom. Black dye can replace the ink sac. Both recipes accept Minecraft's plank tag.

Build with `python3 tools/build.py`. The content and native bundles must both be installed and the native game rebuilt for the 45-slot container to work. Building the packages alone does not install them or change the selected launcher version.
