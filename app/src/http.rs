use crate::http::request::RequestState;
use crate::reload;
use ehttp::Request;
use std::sync::mpsc::channel;

mod request;

#[derive(Default)]
pub struct Http {
    pub logout: RequestState<()>,
}

impl Http {
    pub fn update(&mut self, toasts: &mut egui_notify::Toasts) {
        self.logout.poll_one_off(toasts);

        if let RequestState::Success(_) = self.logout {
            self.logout = RequestState::Idle;
            reload();
        }
    }

    pub fn do_logout(&mut self, ctx: &egui::Context) {
        if matches!(self.logout, RequestState::Idle | RequestState::Error(_)) {
            let (tx, rx) = channel();
            self.logout = RequestState::Loading(rx);

            #[allow(unused_mut)]
            let mut req = Request::post("/api/auth/logout", vec![]);
            #[cfg(target_arch = "wasm32")]
            {
                req.credentials = ehttp::Credentials::SameOrigin;
            }

            let ctx_clone = ctx.clone();
            ehttp::fetch(req, move |result| {
                let parsed = match result {
                    Ok(response) if response.ok => Ok(()),
                    Ok(response) => Err(format!("Server returned HTTP {}", response.status)),
                    Err(err) => Err(err),
                };

                let _ = tx.send(parsed);
                ctx_clone.request_repaint();
            });
        }
    }
}
