use crate::ui::state::sandbox::SandboxState;
use checkmade_core::giga_chess::stockfish::command::SfCommand;
use checkmade_core::giga_chess::stockfish::event::SfEvent;
use checkmade_core::types::session_id::SessionId;
use std::any::TypeId;
use std::collections::HashSet;

#[derive(Clone)]
struct EventBuffer<E: Clone + Send + Sync + 'static> {
    pending: Vec<E>,
    active: Vec<E>,
}

impl<E: Clone + Send + Sync + 'static> Default for EventBuffer<E> {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            active: Vec::new(),
        }
    }
}

impl<E: Clone + Send + Sync + 'static> EventBuffer<E> {
    fn flush(ctx: &egui::Context) {
        ctx.data_mut(|d| {
            let buf: &mut EventBuffer<E> = d.get_temp_mut_or_default(egui::Id::NULL);
            buf.active.clear();
            std::mem::swap(&mut buf.active, &mut buf.pending);
        });
    }

    fn push(ctx: &egui::Context, event: E) {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<Self>(egui::Id::NULL)
                .pending
                .push(event);
        });
    }

    fn any(ctx: &egui::Context) -> bool {
        ctx.data(|d| {
            d.get_temp::<Self>(egui::Id::NULL)
                .is_some_and(|buf| !buf.active.is_empty())
        })
    }

    fn drain(ctx: &egui::Context) -> Vec<E> {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<Self>(egui::Id::NULL)
                .active
                .drain(..)
                .collect()
        })
    }
}

#[derive(Clone, Default)]
struct EventRegistry {
    flushers: Vec<fn(&egui::Context)>,
    registered: HashSet<TypeId>,
}

impl EventRegistry {
    fn register<E: AppEvent + Send + Sync>(&mut self) {
        if self.registered.insert(TypeId::of::<E>()) {
            self.flushers.push(EventBuffer::<E>::flush);
        }
    }
}

pub fn flush_all_events(ctx: &egui::Context) {
    let flushers = ctx.data(|d| {
        d.get_temp::<EventRegistry>(egui::Id::NULL)
            .map(|r| r.flushers.clone())
            .unwrap_or_default()
    });
    for f in flushers {
        f(ctx);
    }
}

pub trait AppEvent: Clone + Send + Sync + 'static + Sized {
    fn send(self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<EventRegistry>(egui::Id::NULL)
                .register::<Self>();
        });
        EventBuffer::push(ctx, self);
    }

    fn fired(ctx: &egui::Context) -> bool {
        EventBuffer::<Self>::any(ctx)
    }

    fn recv(ctx: &egui::Context) -> Vec<Self> {
        ctx.data(|d| {
            d.get_temp::<EventBuffer<Self>>(egui::Id::NULL)
                .map(|buf| buf.active.clone())
                .unwrap_or_default()
        })
    }
}

#[derive(Clone)]
pub struct ErrorEvent(pub String);
impl AppEvent for ErrorEvent {}

#[derive(Clone)]
pub struct InfoEvent(pub String);
impl AppEvent for InfoEvent {}

#[derive(Clone)]
pub struct DisconnectedEvent;
impl AppEvent for DisconnectedEvent {}

#[derive(Clone)]
pub struct ReconnectedEvent;
impl AppEvent for ReconnectedEvent {}

#[derive(Clone)]
pub struct OpenSessionEvent(pub SessionId);
impl AppEvent for OpenSessionEvent {}

#[derive(Clone)]
pub struct OpenSandboxEvent(pub SandboxState);
impl AppEvent for OpenSandboxEvent {}

#[derive(Clone)]
pub struct SfCommandEvent(pub SfCommand);
impl AppEvent for SfCommandEvent {}

#[derive(Clone)]
pub struct SfResponseEvent(pub SfEvent);
impl AppEvent for SfResponseEvent {}
