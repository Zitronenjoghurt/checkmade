use crate::websocket::connection::ConnectionId;
use dashmap::DashMap;
use std::collections::HashSet;
use uuid::Uuid;

pub struct Subscriptions {
    session_users: DashMap<Uuid, HashSet<ConnectionId>>,
    user_sessions: DashMap<ConnectionId, HashSet<Uuid>>,
}

impl Subscriptions {
    pub fn new() -> Self {
        Self {
            session_users: DashMap::new(),
            user_sessions: DashMap::new(),
        }
    }

    pub fn subscribe(&self, session_id: Uuid, conn_id: ConnectionId) {
        self.session_users
            .entry(session_id)
            .or_default()
            .insert(conn_id);
        self.user_sessions
            .entry(conn_id)
            .or_default()
            .insert(session_id);
    }

    pub fn unsubscribe(&self, session_id: Uuid, conn_id: ConnectionId) {
        if let Some(mut conns) = self.session_users.get_mut(&session_id) {
            conns.remove(&conn_id);
        }
        if let Some(mut sessions) = self.user_sessions.get_mut(&conn_id) {
            sessions.remove(&session_id);
        }
    }

    pub fn unsubscribe_all(&self, conn_id: ConnectionId) {
        if let Some((_, sessions)) = self.user_sessions.remove(&conn_id) {
            for session_id in &sessions {
                if let Some(mut conns) = self.session_users.get_mut(session_id) {
                    conns.remove(&conn_id);
                }
            }
        }
    }

    pub fn with_subscribers<F, R>(&self, session_id: &Uuid, f: F) -> R
    where
        F: FnOnce(&HashSet<ConnectionId>) -> R,
    {
        match self.session_users.get(session_id) {
            Some(conns) => f(&conns),
            None => f(&HashSet::new()),
        }
    }

    pub fn get_subscribers(&self, session_id: &Uuid) -> Vec<ConnectionId> {
        self.session_users
            .get(session_id)
            .map(|conns| conns.iter().copied().collect())
            .unwrap_or_default()
    }
}
