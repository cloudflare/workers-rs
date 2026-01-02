use crate::{
    error::Error,
    helpers::IntoResponse,
    message::SendData,
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
    storage: Storage,
    /// Holds multiple clients' connections used to broadcast messages.
    users: Users,
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
    pub fn add(&self, user: User) -> Result<(), Error> {
        self.users.add(user)?;
        Ok(())
    }

    /// Broadcasts a message to all connected clients.
    pub fn broadcast(&self, data: &SendData) {
        self.users.broadcast(data);
    }

    /// Removes a user corresponding to the id.
    pub fn remove(&self, id: &str) {
        self.users.remove(id);
    }

    pub const fn storage(&self) -> &Storage {
        &self.storage
    }

    pub const fn users(&self) -> &Users {
        &self.users
    }

    /// Update online users' count.
    pub async fn update(&self, ops: Update) -> Result<u64, Error> {
        // view increment
        let count = self.storage.update(ops);
        count.await
    }
}
