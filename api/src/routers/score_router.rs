use axum::{
    Json, Router,
    body::Body,
    extract::{Query, State},
    response::Response,
    routing::{get, post},
};
use common::{errors::CommonError, model::CommonResult};

use crate::{AppState, application::score, model::score::ExportExcel};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/score",
        Router::new()
            .route("/export-excel", get(export_excel))
            .route("/calculate-monthly", post(calculate_bonus))
            .route("/send-monthly-email", post(send_monthly_email)),
    )
}

async fn export_excel(
    state: State<AppState>,
    Query(params): Query<ExportExcel>,
) -> Result<Response<Body>, CommonError> {
    let export = score::export_excel_data(&state, &params)
        .await
        .map_err(|e| CommonError::InvalidInput(e.to_string()))?;

    let resp = Response::builder()
        .header(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("Content-Disposition", export.content_disposition)
        .body(Body::from(export.file_data))
        .unwrap();
    Ok(resp)
}

async fn calculate_bonus(state: State<AppState>) -> Result<Json<CommonResult<()>>, CommonError> {
    score::calculate_bonus(&state)
        .await
        .map_err(|e| CommonError::InvalidInput(e.to_string()))?;
    Ok(Json(CommonResult::success(None)))
}

async fn send_monthly_email(state: State<AppState>) -> Result<Json<CommonResult<()>>, CommonError> {
    score::send_monthly_email(state)
        .await
        .map_err(|e| CommonError::InvalidInput(e.to_string()))?;
    Ok(Json(CommonResult::success(None)))
}
