// SPDX-License-Identifier: GPL-3.0-or-later
#[derive(Clone, Debug)]
pub struct Fairy {
    pub enabled: bool,
    pub racial_glider: bool,
    pub racial_glide_armor_blocked: bool,
    pub oni_ending: bool,
    pub used: bool,
    pub previous_jump: bool,
    pub ascending: bool,
    pub slowfall: bool,
    pub parasol: bool,
    pub parasol_sent: Option<bool>,
    pub parasol_revision: u64,
    pub potion_slowfall: bool,
    pub previous_velocity: f64,
    pub size: f32,
    pub sound: Option<&'static str>,
    pub character_health: f32,
    pub kit_checked: Option<(String, i32)>,
}
impl Default for Fairy {
    fn default() -> Self {
        Self {
            enabled: false,
            racial_glider: false,
            racial_glide_armor_blocked: false,
            oni_ending: false,
            used: false,
            previous_jump: false,
            ascending: false,
            slowfall: false,
            parasol: false,
            parasol_sent: None,
            parasol_revision: 0,
            potion_slowfall: false,
            previous_velocity: 0.0,
            size: 1.0,
            sound: None,
            character_health: 20.0,
            kit_checked: None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Input {
    pub jump: bool,
    pub grounded: bool,
    pub blocked: bool,
    pub velocity: f64,
}
impl Fairy {
    pub fn step(&mut self, input: Input) -> bool {
        let pressed = input.jump && !self.previous_jump;
        self.previous_jump = input.jump;
        if pressed && self.parasol {
            if input.grounded { self.used = false; }
            self.parasol = false;
            self.sound = Some("entity.fairy.cancel");
            self.previous_velocity = input.velocity;
            return false;
        }
        if !self.enabled || input.grounded || input.blocked {
            self.used = false;
            self.ascending = false;
            self.slowfall = false;
            self.previous_velocity = input.velocity;
            return false;
        }
        let boost = pressed && !self.used;
        if boost {
            self.used = true;
            self.slowfall = false;
            self.ascending = true;
            self.sound = Some("entity.fairy.jump");
        } else if pressed && self.slowfall {
            self.slowfall = false;
            self.ascending = false;
            self.sound = Some("entity.fairy.cancel");
        } else if self.ascending && input.velocity <= self.previous_velocity {
            self.ascending = false;
            self.slowfall = true;
        }
        self.previous_velocity = input.velocity + if boost { 12.0 / 20.0 } else { 0.0 };
        boost
    }
}
