use crate::{
    error::Error,
    message::{Message, WsMessage},
    route::Route,
    storage::Storage,
    user::{User, UserInfo},
};

use std::{collections::HashMap, rc::Rc, sync::Mutex};
use worker::{
    async_trait, durable_object, js_sys, wasm_bindgen, wasm_bindgen_futures, worker_sys, Env,
    Request, Response,
};

pub enum Update {
    Decrease,
    Increase,
}

pub type Users = HashMap<String, User>;

#[durable_object]
#[derive(Clone)]
pub struct LiveCounter {
    /// Struct wrapper for [worker::State].
    storage: Storage,
    /// Holds multiple clients' connections used to broadcast messages.
    users: Rc<Mutex<Users>>,
}

#[durable_object]
impl DurableObject for LiveCounter {
    fn new(state: State, _: Env) -> Self {
        Self {
            storage: Storage::new(state),
            users: Rc::new(Mutex::new(Users::new())),
        }
    }

    // this will be called using its stub (fetch_with_request or fetch_with_str)
    // note - it is not to be called directly.
    async fn fetch(&mut self, req: Request) -> Result<Response, worker::Error> {
        match Route::new(req.url()?.path_segments().unwrap()) {
            Route::Counter => Ok(Route::websocket(self.clone(), &req).await?),
            Route::InvalidRoute => Ok(Route::invalid(req)?),
        }
    }
}

impl LiveCounter {
    /// Broadcasts a message to all connected clients.
    pub fn broadcast(&self, msg: &Message) -> Result<(), Error> {
        // iterates connected clients to send the message
        for (id, session) in &*self.users.lock()? {
            if session.session.send_with_str(&msg.to_string()).is_err() {
                self.remove(id)?;
            }
        }
        Ok(())
    }

    /// Add a new user to the session.
    pub fn add(&self, user: User) -> Result<(), Error> {
        self.users.lock()?.insert(user.info.id.clone(), user);
        Ok(())
    }

    /// Removes a user corresponding to the id.
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        self.users.lock()?.remove(id);
        Ok(())
    }

    /// Update online users' count.
    pub async fn update(&self, ops: Update, info: &UserInfo) -> Result<(), Error> {
        // view increment
        let count = self.storage.update(ops).await?;
        // broadcast to connected clients
        self.broadcast(&Message::new(info, &WsMessage::Conn(Some(count))))?;
        Ok(())
    }
}
