# Magic

The normal survival inventory's book button opens Magic. Click a spell to cast;
drag it to another slot to rearrange it. The book returns to the real inventory.
Armor, offhand and crafting are locked on this page. Spells are virtual permanent
unlocks, never item stacks, so dropping, crafting and container transfers cannot
consume or duplicate them. Workstations retain their normal recipe books.

`assets/lunatic/horizon/characters.json` supplies `starting_spells` immediately after
`starting_items`. Entries are spell IDs. Every approved character's spells are added
to that player's collection in the world; switching characters never revokes them.
The layout is saved in player NBT under `horizon:magic`, with revision-checked swaps.
Unreadable saved magic data is preserved and made read-only.

`assets/lunatic/horizon/magic.json` defines icons, name colors, hunger costs, scale,
phase durations (20 ticks/second), effect markers and Giant/Tiny behavior. The host
loads this only from enabled content. Requests contain only an unlocked ID or slot
indices; the server decides costs, timing and attributes. Casting closes the menu,
requires sufficient hunger, and allows one size spell at a time. Both survival and
creative casting pay the defined hunger cost.

Suika unlocks Gigantification (4-second growth, 20-second Giant, 4-second return;
6 hunger points) and Miniaturization (one-fifth size; 3-second shrink, 30-second Tiny, 3-second
return; 4 hunger points). Size multiplies her normal character size, including
model, hitbox and eye height. Growth is strongly slowed. Giant adds six melee
damage; its landings after an airborne jump cause vanilla power-3 explosions,
with a six-block damage radius, caster immunity, resistance-aware compact craters,
normal drops and no fire. Tiny adds 50% movement speed, a 0.3 mining multiplier
(Mining Fatigue I), +0.2 jump strength (Jump Boost II), and fall-damage immunity.
Giant reach tracks the extra eye height through the size transition, covering melee, breaking and interaction; it restores at normal size. Growth clears destructible cells before expanding the hitbox and emits at most one full explosion per half-second when obstructed. Other bonuses end with their named active phase.

Growth/return use eased scaling, sparse orange/blue ribbon particles and pitched
amethyst chimes. Full Giant announces itself with a dragon growl; landing uses
vanilla explosion audio/particles. Returning finishes with a soft item-frame pop.
All size/stat modifiers are transient and cleaned on completion and initialization;
rejoining or respawning restores normal character size without losing unlocks.

Slot moves predict immediately and reconcile against revision acknowledgements; queued moves remain visible while prior swaps are acknowledged. Spell tooltips show a HUD-sized food icon, cost and the content-defined description.
