use mod_api::{
    ClientHost, ClientResourcesReloadedEvent, ClientSetupEvent, Hooks, LoopContext, ModContainer,
    PlaySoundEvent, Registrar, RegistrationContext,
};
use mod_api::{ClientDisconnectingEvent, HostSide, IPayloadContext, IPayloadHandler, PayloadBytes, PayloadHandleError, PayloadId, PlayerLoggedOutEvent, PlayerRef, PlayerTickPostEvent, ServerHost, ServerStoppingEvent};
use std::{cell::RefCell, collections::VecDeque, sync::Arc};
use std::sync::atomic::{AtomicBool, Ordering};

const DONATION_BOX: &str = "lunatic:donation_box";
const DONATION_BOX_SLOTS: usize = 45;
const PLAYER_DEATH: &str = "lunatic:entity.player.death";

const CLIENT_REQUESTS: &[(&str, usize)] = &[
    ("lunatic:rod_fire", 9),
    ("lunatic:gourd_attack", 8),
    ("lunatic:parasol_fire", 2),
];
const SERVER_REPLIES: &[(&str, usize)] = &[
    ("lunatic:gourd_tethers", 64 * 1024),
    ("lunatic:orb_bursts", 64 * 1024),
    ("lunatic:parasol_state", 1),
];

type Request = (PlayerRef, &'static str, Vec<u8>);
type Reply = (&'static str, Vec<u8>);

thread_local! {
    static REQUESTS: RefCell<VecDeque<Request>> = const { RefCell::new(VecDeque::new()) };
    static REPLIES: RefCell<VecDeque<Reply>> = const { RefCell::new(VecDeque::new()) };
}

pub fn take_requests() -> VecDeque<Request> {
    REQUESTS.with_borrow_mut(std::mem::take)
}

pub fn take_replies() -> VecDeque<Reply> {
    REPLIES.with_borrow_mut(std::mem::take)
}

struct RequestHandler {
    channel: &'static str,
    limit: usize,
}

impl IPayloadHandler for RequestHandler {
    fn handle(&self, bytes: PayloadBytes, ctx: &mut dyn IPayloadContext) -> Result<(), PayloadHandleError> {
        if bytes.0.len() <= self.limit && let Ok(player) = ctx.player() {
            REQUESTS.with_borrow_mut(|queue| {
                let player_pending = queue.iter().filter(|(owner, _, _)| *owner == player).count();
                if queue.len() < 256 && player_pending < 16 {
                    queue.push_back((player, self.channel, bytes.0));
                }
            });
        }
        Ok(())
    }
}

struct ReplyHandler {
    channel: &'static str,
    limit: usize,
}

impl IPayloadHandler for ReplyHandler {
    fn handle(&self, bytes: PayloadBytes, _: &mut dyn IPayloadContext) -> Result<(), PayloadHandleError> {
        if bytes.0.len() <= self.limit {
            REPLIES.with_borrow_mut(|queue| {
                if queue.len() == 32 {
                    queue.pop_front();
                }
                queue.push_back((self.channel, bytes.0));
            });
        }
        Ok(())
    }
}

#[derive(Default)]
struct Lunatic {
    player_death_sound_available: AtomicBool,
}

impl Lunatic {
    fn check_death_sound(&self, host: &dyn ClientHost) {
        let available = host
            .sound_events()
            .iter()
            .any(|id| id.namespace() == "lunatic" && id.path() == "entity.player.death");
        self.player_death_sound_available
            .store(available, Ordering::Relaxed);
    }
}

#[mod_api::horizon_mod]
impl Hooks for Lunatic {
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        let mut registration = registrar.payload_handlers();
        let mut payloads = registration.registrar("1").optional();
        for &(channel, limit) in CLIENT_REQUESTS {
            payloads.play_to_server(PayloadId(channel.into()), Arc::new(RequestHandler { channel, limit }));
        }
        for &(channel, limit) in SERVER_REPLIES {
            payloads.play_to_client(PayloadId(channel.into()), Some(Arc::new(ReplyHandler { channel, limit })));
        }
        registrar
            .block_entity_type("donation_box")
            .blocks(&[DONATION_BOX])
            .build();
        let block = registrar.block_key(DONATION_BOX);
        let slots: Vec<_> = (0..DONATION_BOX_SLOTS).collect();
        registrar.block_container(
            &block,
            ModContainer {
                slots: DONATION_BOX_SLOTS,
                faces: std::array::from_fn(|_| slots.clone()),
                inputs: slots.clone(),
                outputs: slots,
                menu: Some("minecraft:generic_9x5".into()),
            },
        );
    }
    fn player_tick_post(&self, _: HostSide<'_>, _: &mut PlayerTickPostEvent) {
        // Subscribe to the player turn. The replacement runs here with the
        // engine's existing character, inventory, and replay state in scope.
    }

    fn player_logged_out(&self, _: &mut dyn ServerHost, event: &mut PlayerLoggedOutEvent<'_>) {
        REQUESTS.with_borrow_mut(|queue| queue.retain(|(player, _, _)| player.uuid() != event.player));
    }

    fn server_stopping(&self, _: &mut dyn ServerHost, _: &mut ServerStoppingEvent) {
        REQUESTS.with_borrow_mut(|queue| queue.clear());
    }

    fn client_disconnecting(&self, _: &dyn ClientHost, _: &mut ClientDisconnectingEvent) {
        REPLIES.with_borrow_mut(|queue| queue.clear());
    }

    fn client_setup(&self, host: &mut dyn ClientHost, _: &mut ClientSetupEvent) {
        self.check_death_sound(host);
    }

    fn client_resources_reloaded(
        &self,
        host: &mut dyn ClientHost,
        _: &mut ClientResourcesReloadedEvent,
    ) {
        self.check_death_sound(host);
    }

    fn play_sound(&self, _: &dyn LoopContext, event: &mut PlaySoundEvent<'_>) {
        if !self.player_death_sound_available.load(Ordering::Relaxed) {
            return;
        }
        let Some(sound) = event.sound.as_mut() else {
            return;
        };
        match sound.identifier.as_str() {
            "minecraft:entity.player.death" | "entity.player.death" => {
                sound.identifier = PLAYER_DEATH.into();
            }
            PLAYER_DEATH => {}
            _ => return,
        }
        sound.pitch = 1.0;
    }
}

pub mod fairy_state;
pub mod gourd_physics;
pub mod projectile_state;

pub mod chest_physics;
