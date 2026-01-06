use axum::extractors::{State, Path};
use crate::App;
use crate::resources::foos::model::Foo;

#[worker::send]
pub async fn foo_api(State(state): State<App>, Path(foo_id): Path(String)) -> Result<axum::http::Response<Foo>> {
    let foo = state.foo_service.get(foo_id).await
}
