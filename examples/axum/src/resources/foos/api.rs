use crate::error::AppError;
use crate::{resources::foos::model::Foo, AppState};
use axum::extract::{Path, State};
use axum::Json;
use axum_macros::debug_handler;
use worker::Result;

/// `get()` requires the `#[worker::send]` macro because Cloudflare Workers
/// execute a handler's future on a single JavaScript event loop.
///
/// The macro helps make `await` boundaries in the handler's function body `Send`
/// so the worker runtime can safely poll them.
///
/// You can read more about it here in the "`Send` Helpers" section:
/// https://docs.rs/worker/latest/worker/
#[worker::send]
#[debug_handler]
pub async fn get(
    State(state): State<AppState>,
    Path(foo_id): Path<String>,
) -> Result<Json<Foo>, AppError> {
    let foo = state.foo_service.get(foo_id).await?;

    foo.ok_or(AppError::NotFound).map(Json)
}
