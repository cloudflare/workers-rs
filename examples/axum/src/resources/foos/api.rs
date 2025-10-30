use crate::error::AppError;
use crate::{resources::foos::model::Foo, AppState};
use axum::extract::{Path, State};
use axum::Json;
use axum_macros::debug_handler;
use worker::Result;

#[debug_handler]
pub async fn get(
    State(state): State<AppState>,
    Path(foo_id): Path<String>,
) -> Result<Json<Foo>, AppError> {
    let foo = state.foo_service.get(foo_id).await?;

    foo.ok_or(AppError::NotFound).map(Json)
}
