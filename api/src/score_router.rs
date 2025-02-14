use std::io::Cursor;

use axum::{
    body::Body,
    extract::{Query, State},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use chrono::{Datelike, Utc};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rust_xlsxwriter::Workbook;
use sea_orm::{Set, TryIntoModel};

use common::{date::get_last_month, errors::CommonError, model::CommonResult};
use entity::monthly_score;
use service::model::score::{load_score_strategy, CommonScore, ScoreDto};

use crate::{model::score::ExportExcel, AppState};

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/score",
        Router::new()
            .route("/export-excel", get(export_excel))
            .route("/calculate-monthly", post(calculate_bonus)),
    )
}

async fn export_excel(
    state: State<AppState>,
    Query(params): Query<ExportExcel>,
) -> Result<Response<Body>, CommonError> {
    let monthly_records = state
        .score_stg()
        .list_score_by_month(params.year, params.month)
        .await
        .unwrap();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_column_width(0, 22).unwrap();
    worksheet.set_column_width(1, 22).unwrap();

    let mut row_idx = 1;

    worksheet.write_string(0, 0, "姓名").unwrap();
    worksheet.write_string(0, 1, "金额(元)").unwrap();

    for score in monthly_records {
        if score.exchanged != 0 {
            worksheet
                .write_string(row_idx as u32, 0, score.student_name)
                .unwrap();
            worksheet.write(row_idx as u32, 1, score.exchanged).unwrap();
            row_idx += 1;
        }
    }
    let file_name = format!(
        "开源实习 临时用工人员劳务费上报表-{}年{}月.xlsx",
        params.year, params.month
    );

    let encoded_filename = utf8_percent_encode(&file_name, NON_ALPHANUMERIC).to_string();
    let disposition = format!("attachment; filename*=UTF-8''{}", encoded_filename);

    let mut buffer = Cursor::new(Vec::new());

    workbook.save_to_writer(&mut buffer).unwrap();
    let file_data = buffer.into_inner();

    let resp = Response::builder()
        .header(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("Content-Disposition", disposition)
        .body(Body::from(file_data))
        .unwrap();
    Ok(resp)
}

#[axum::debug_handler]
async fn calculate_bonus(state: State<AppState>) -> Result<Json<CommonResult<()>>, CommonError> {
    let now = Utc::now().naive_utc();
    let calculate_month = get_last_month(now.into());

    // 获取上个月全部记录
    let monthly_records = state
        .score_stg()
        .list_score_by_month(calculate_month.year(), calculate_month.month() as i32)
        .await
        .unwrap();

    for model in &monthly_records {
        let sum = model.carryover_score + model.new_score;
        let student = state
            .student_stg()
            .get_student_by_login(&model.github_login)
            .await
            .unwrap();
        let consume_score = {
            let strategy = if let Some(student) = student {
                load_score_strategy(student, calculate_month)
            } else {
                tracing::error!("Invalid Student Status:{}", model.github_login);
                // fallback to default rule
                Box::new(CommonScore)
            };
            strategy.consumed_score(sum)
        };
        let mut a_model: monthly_score::ActiveModel = model.clone().into();
        // 更新上个月的发放情况
        a_model.consumption_score = Set(consume_score);
        a_model.exchanged = Set(consume_score * 50);
        a_model.update_at = Set(Utc::now().naive_utc());
        state
            .score_stg()
            .update_score(a_model.clone())
            .await
            .unwrap();
        // 更新本月的上月结转分数
        let last_month: ScoreDto = a_model.try_into_model().unwrap().into();
        state
            .score_stg()
            .insert_or_update_carryover_score(last_month)
            .await
            .unwrap();
    }
    Ok(Json(CommonResult::success(None)))
}
