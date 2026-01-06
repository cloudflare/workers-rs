
struct FooService {
    kv: KvStore
}

impl FooService {
    pub fn new(kv: KvStore) -> FooService {
        FooService {
            kv
        }
    }

    pub async fn get(foo_id: String) -> Option<Foo> {
        if let Some(q) = self.cache.get::<Foo>(&foo_id).await? {
            return Ok(q);
        }
        None
    }
}
