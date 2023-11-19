use crate::{
    error::Error,
    helpers::IntoResponse,
    message::MsgResponse,
    route::Route,
    storage::{Storage, Update},
    user::{User, Users},
};

use worker::{
    async_trait, durable_object, js_sys, wasm_bindgen, wasm_bindgen_futures, worker_sys, Env,
    Request, Response,
};

#[durable_object]
#[derive(Clone)]
pub struct LiveCounter {
    /// Struct wrapper for [worker::State].
    pub(crate) storage: Storage,
    /// Holds multiple clients' connections used to broadcast messages.
    pub(crate) users: Users,
}

#[durable_object]
impl DurableObject for LiveCounter {
    fn new(state: State, _: Env) -> Self {
        Self {
            storage: Storage::new(state),
            users: Users::default(),
        }
    }

    // this will be called using its stub (fetch_with_request or fetch_with_str)
    // note - it is not to be called directly.
    async fn fetch(&mut self, req: Request) -> worker::Result<Response> {
        match Route::new(&req) {
            Ok(route) => match route {
                Route::Chat(info) => Route::websocket(self.clone(), info).await,
                Route::InvalidRoute => Route::invalid(),
            }
            .into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl LiveCounter {
    /// Add a new user to the session.
    pub(crate) fn add(&self, user: User) -> Result<(), Error> {
        self.users.add(user)?;
        Ok(())
    }

    /// Broadcasts a message to all connected clients.
    pub(crate) fn broadcast(&self, msg: &MsgResponse) {
        self.users.broadcast(msg);
    }

    /// Removes a user corresponding to the id.
    pub(crate) fn remove(&self, id: &str) {
        self.users.remove(id);
    }

    /// Update online users' count.
    pub(crate) async fn update(&self, ops: Update) -> Result<u64, Error> {
        // view increment
        let count = self.storage.update(ops);
        count.await
    }
}
