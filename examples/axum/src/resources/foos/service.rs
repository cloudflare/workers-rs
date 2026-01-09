use crate::resources::foos::model::Foo;
use worker::{Result, KvStore, KvError};

pub struct FooService {
    kv: KvStore
}

impl FooService {
    pub fn new(kv: KvStore) -> FooService {
        FooService {
            kv
        }
    }

    pub async fn get(&self, foo_id: String) -> Result<Option<Foo>, KvError> {
        let maybe_foo = self.kv.get(&foo_id);
        maybe_foo.json::<Foo>().await
    }
}
