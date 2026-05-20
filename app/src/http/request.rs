use std::sync::mpsc::Receiver;

pub enum RequestState<T> {
    Idle,
    Loading(Receiver<Result<T, String>>),
    Success(T),
    Error(String),
}

#[allow(clippy::derivable_impls)]
impl<T> Default for RequestState<T> {
    fn default() -> Self {
        Self::Idle
    }
}

impl RequestState<()> {
    pub fn poll_one_off(&mut self, toasts: &mut egui_notify::Toasts) {
        if let RequestState::Loading(rx) = self
            && let Ok(response) = rx.try_recv()
        {
            *self = match response {
                Ok(_) => RequestState::Success(()),
                Err(err) => {
                    toasts.error(&err);
                    RequestState::Error(err)
                }
            };
        }
    }
}
