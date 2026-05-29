#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

    pub struct StockfishEngine {
        worker: Worker,
        incoming: Rc<RefCell<VecDeque<String>>>,
    }

    impl StockfishEngine {
        pub fn new() -> Self {
            let opts = WorkerOptions::new();
            opts.set_type(WorkerType::Module);

            let worker = Worker::new_with_options("./stockfish/stockfish-18-lite.js", &opts)
                .expect("failed to spawn stockfish worker");

            let incoming: Rc<RefCell<VecDeque<String>>> = Rc::default();
            let buf = incoming.clone();

            let cb = Closure::<dyn Fn(MessageEvent)>::wrap(Box::new(move |e: MessageEvent| {
                if let Some(msg) = e.data().as_string() {
                    buf.borrow_mut().push_back(msg);
                }
            }));
            worker.set_onmessage(Some(cb.as_ref().unchecked_ref()));
            cb.forget();

            Self { worker, incoming }
        }
    }

    impl crate::engine::Engine for StockfishEngine {
        fn send(&self, cmd: &str) {
            self.worker
                .post_message(&JsValue::from_str(cmd))
                .expect("post_message failed");
        }

        fn drain(&self) -> Vec<String> {
            self.incoming.borrow_mut().drain(..).collect()
        }
    }

    impl Drop for StockfishEngine {
        fn drop(&mut self) {
            let _ = self.worker.post_message(&JsValue::from_str("quit"));
            self.worker.terminate();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::StockfishEngine;
