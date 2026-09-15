use crate::error::Error;

use std::rc::Rc;
use worker::State;

/// Storage's key.
const ONLINE_USERS: &str = "ONLINE_USERS";

pub enum Update {
    Decrease,
    Increase,
}

#[derive(Clone)]
// Rc implements Clone and makes life easier
pub struct Storage(Rc<State>);

impl Storage {
    pub fn new(state: State) -> Self {
        Self(Rc::new(state))
    }

    /// Returns the number of online users.
    async fn get_count(&self) -> u64 {
        let storage = self.0.storage();
        let count = storage.get(ONLINE_USERS);

        // it returns Err(JsError("No such value in storage.")) if it didn't exist before
        // return 0 instead for first time initialization
        count.await.unwrap_or(0)
    }

    /// Update online users' count.
    pub async fn update(&self, ops: Update) -> Result<u64, Error> {
        let mut count = self.get_count().await;
        match ops {
            Update::Decrease => count -= 1,
            Update::Increase => count += 1,
        };
        let mut storage = self.0.storage();
        storage.put(ONLINE_USERS, count).await?;

        Ok(count)
    }
}
