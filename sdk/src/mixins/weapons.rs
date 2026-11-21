// SPDX-License-Identifier: GPL-3.0-or-later
use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
use crate::game::lunatic_races::GameLunaticRacesExt as _;
use super::{Game, lunatic_gourd_motion as gourd_motion, lunatic_projectile_motion as lunatic_motion, lunatic_yuuka_flight as lunatic_yuuka};
use horizon_client::{
    projectiles::{Projectile, thrown_type},
    text::Text,
};
use horizon_core::geometry::Vec3;

pub(super) const ONI_COLOR: u32 = 0xffffaa00;
pub(super) const YUUKA_COLOR: u32 = 0xff00aa00;
const DESCRIPTION_COLOR: u32 = 0xffaaaaaa;

#[derive(Clone, Copy)]
pub(super) struct Sound {
    event: &'static str,
    gain: f32,
}

impl Sound {
    pub(super) const fn new(event: &'static str, gain: f32) -> Self {
        Self { event, gain }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Action {
    pub(super) wear: u32,
    pub(super) race: Option<&'static str>,
    sound: Option<Sound>,
}

impl Action {
    const NONE: Self = Self {
        race: None,
        wear: 0,
        sound: None,
    };

    const fn for_race(self, race: &'static str) -> Self {
        Self {
            race: Some(race),
            ..self
        }
    }

    const fn wearing(self, wear: u32) -> Self {
        Self { wear, ..self }
    }

    const fn sounding(event: &'static str, gain: f32) -> Self {
        Self {
            sound: Some(Sound::new(event, gain)),
            ..Self::NONE
        }
    }
}

type Tooltip = fn(&Weapon, &mut Vec<(Text, u32)>);

pub(super) struct Weapon {
    pub(super) item: &'static str,
    pub(super) color: u32,
    pub(super) primary: Action,
    pub(super) alternate: Action,
    pub(super) passive: Action,
    pub(super) ignores_hurt_cooldown: bool,
    pub(super) tooltip: Option<Tooltip>,
}

impl Weapon {
    const fn new(item: &'static str, color: u32) -> Self {
        Self {
            item,
            color,
            primary: Action::NONE,
            alternate: Action::NONE,
            passive: Action::NONE,
            ignores_hurt_cooldown: true,
            tooltip: None,
        }
    }

    pub(super) fn attack(&self, alternate: bool) -> Action {
        if alternate {
            self.alternate
        } else {
            self.primary
        }
    }
}

pub(super) const PURIFICATION_ROD: Weapon = Weapon {
    primary: Action::sounding("item.purification_rod.activate", 0.6),
    ..Weapon::new("lunatic:purification_rod", 0xffff5555)
};
pub(super) const ROD_NO_TARGET: Sound = Sound::new("item.purification_rod.no_target", 0.6);

pub(super) const LAMPAD_TORCH: Weapon = Weapon {
    primary: Action::sounding("item.lampad_torch.shoot", 0.7),
    alternate: Action::sounding("item.lampad_torch.shoot", 0.7),
    ..Weapon::new("lunatic:lampad_torch", 0xffaa00aa)
};

pub(super) const IBUKI_GOURD: Weapon = Weapon {
    primary: Action::sounding("item.gourd.fire", 0.4).wearing(3),
    tooltip: Some(gourd_tooltip),
    ..Weapon::new("lunatic:ibuki_gourd", 0xffff55ff)
};
pub(super) const GOURD_REEL: Sound = Sound::new("item.gourd.reel", 0.8);

pub(super) const YUUKA_PARASOL: Weapon = Weapon {
    primary: Action::sounding("item.yuuka_parasol.fire", 0.55)
        .wearing(3)
        .for_race("yuuka"),
    alternate: Action::sounding("item.yuuka_parasol.fire", 0.55)
        .wearing(3)
        .for_race("yuuka"),
    passive: Action::sounding("item.yuuka_parasol.open", 0.5).wearing(3),
    tooltip: Some(parasol_tooltip),
    ..Weapon::new("lunatic:yuuka_parasol", 0xff55ff55)
};

pub(super) const KOISHI_KNIFE: Weapon = Weapon {
    tooltip: Some(|_, lines| {
        lines.push((Text::literal("Stone sword damage, swing cooldown is 50% lower, 25% chance to make enemies bleed."), DESCRIPTION_COLOR));
    }),
    ..Weapon::new("lunatic:koishi_knife", 0xff55ff55)
};

pub(super) fn weapon(item: &str) -> Option<&'static Weapon> {
    [
        &PURIFICATION_ROD,
        &LAMPAD_TORCH,
        &IBUKI_GOURD,
        &YUUKA_PARASOL,
        &KOISHI_KNIFE,
    ]
    .into_iter()
    .find(|weapon| weapon.item == item)
}

pub(super) fn projectile_weapon(projectile: &Projectile) -> Option<&'static Weapon> {
    if lunatic_motion::variant(projectile).is_some() {
        Some(&PURIFICATION_ROD)
    } else if lunatic_yuuka::ours(projectile) {
        Some(&YUUKA_PARASOL)
    } else if gourd_motion::ours(projectile) {
        Some(&IBUKI_GOURD)
    } else if projectile.kind.id().starts_with("lunatic:lampad_") {
        Some(&LAMPAD_TORCH)
    } else {
        None
    }
}

pub(super) fn description(prefix: &str, label: &str, color: u32, suffix: &str) -> (Text, u32) {
    let text = Text::colored(prefix, DESCRIPTION_COLOR)
        .append(Text::colored(label, color))
        .append(Text::colored(suffix, DESCRIPTION_COLOR));
    (text, u32::MAX)
}

fn gourd_tooltip(_: &Weapon, lines: &mut Vec<(Text, u32)>) {
    lines.push(description("Primary fire for ", "Oni", ONI_COLOR,
        ": Hurl the gourd up to 16 blocks away as a flail, dealing great damage. Left click to pull back in. Cooldown: 4s"));
    lines.push(description("Alt fire: Get drunk. For ", "Oni", ONI_COLOR,
        ": Gain super strength for 12-20 seconds before drunkenness sets in, empowering melee damage and providing explosive punches. Effect expiry consumes 2-5 hunger."));
}

fn parasol_tooltip(_: &Weapon, lines: &mut Vec<(Text, u32)>) {
    use super::lunatic_parasol::FALL_DISTANCE;
    let primary = thrown_type(lunatic_yuuka::PRIMARY).map_or(6.0, |kind| kind.damage);
    let alternate = thrown_type(lunatic_yuuka::FLOWERS[0]).map_or(5.0, |kind| kind.damage);
    let range = lunatic_yuuka::PRIMARY_RANGE;
    lines.push(description(
        "Primary fire as ",
        "Yuuka",
        YUUKA_COLOR,
        &format!(": Fire a flower bullet volley. Each bullet deals {primary} damage and travels up to {range} blocks."),
    ));
    lines.push(description(
        "Alt fire as ",
        "Yuuka",
        YUUKA_COLOR,
        &format!(": Fire a stream of large flower bullets in front of you. Each bullet deals {alternate} damage."),
    ));
    lines.push(description(
        "",
        "Passive",
        ONI_COLOR,
        &format!(": When falling more than {FALL_DISTANCE} blocks while holding the parasol, activate slow fall."),
    ));
}

impl GameLunaticWeaponsExt for Game<'_> {
     fn lunatic_worn_stack(
        &mut self,
        stack: &horizon_client::items::ItemStack,
        amount: u32,
        hand: horizon_core::equipment_slot::EquipmentSlot,
    ) -> Option<horizon_client::items::ItemStack> {
        let mut next = stack.clone();
        if self.creative || amount == 0 {
            return Some(next);
        }
        let max = self
            .items()
            .get(&stack.item)
            .map_or(0, |item| item.max_damage);
        if max == 0 {
            return Some(next);
        }
        next.damage = next.damage.saturating_add(self.wear(stack, amount));
        if next.damage >= max {
            self.award_stat("broken", &next.item, 1);
            self.player_equipment_broke(hand, &next);
            None
        } else {
            Some(next)
        }
    }

     fn lunatic_wear_weapon(&mut self, weapon: &Weapon, action: Action, offhand: bool) {
        use horizon_core::equipment_slot::EquipmentSlot;
        let (slot, hand) = if offhand {
            (super::shield::OFFHAND_SLOT, EquipmentSlot::OffHand)
        } else {
            (self.selected, EquipmentSlot::MainHand)
        };
        if let Some(stack) = self
            .stack_in(slot)
            .filter(|s| s.item.as_ref() == weapon.item)
            .cloned()
        {
            *self.stack_in_mut(slot) = self.lunatic_worn_stack(&stack, action.wear, hand);
        }
    }

     fn lunatic_action_allowed(&self, action: Action) -> bool {
        action.race.is_none_or(|race| self.lunatic_has_race(race))
    }

     fn lunatic_weapon_sound(&mut self, sound: Sound, at: Vec3) {
        self.lunatic_sound(sound.event, at, sound.gain);
    }

     fn lunatic_action_sound(&mut self, action: Action, at: Vec3) {
        if let Some(sound) = action.sound {
            self.lunatic_weapon_sound(sound, at);
        }
    }
}

pub trait GameLunaticWeaponsExt {
    fn lunatic_worn_stack(
        &mut self,
        stack: &horizon_client::items::ItemStack,
        amount: u32,
        hand: horizon_core::equipment_slot::EquipmentSlot,
    ) -> Option<horizon_client::items::ItemStack>;
    fn lunatic_wear_weapon(&mut self, weapon: &Weapon, action: Action, offhand: bool);
    fn lunatic_action_allowed(&self, action: Action) -> bool;
    fn lunatic_weapon_sound(&mut self, sound: Sound, at: Vec3);
    fn lunatic_action_sound(&mut self, action: Action, at: Vec3);
}
