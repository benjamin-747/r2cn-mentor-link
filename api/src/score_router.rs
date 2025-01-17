use axum::{extract::State, routing::post, Json, Router};
use chrono::{Datelike, Utc};
use common::{date::get_last_month, errors::CommonError, model::CommonResult};
use rust_xlsxwriter::Workbook;

use crate::AppState;

pub fn routers() -> Router<AppState> {
    Router::new().nest(
        "/score",
        Router::new()
            .route("/excel", post(export_excel))
            .route("/calculate-monthly", post(calculate_bonus)),
    )
}

async fn export_excel(state: State<AppState>) -> Result<(), CommonError> {
    let now = Utc::now();

    let month_score = state
        .score_stg()
        .list_score_by_month(now.year(), now.month() as i32)
        .await
        .unwrap();

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.set_column_width(0, 22).unwrap();
    worksheet.set_column_width(1, 22).unwrap();

    let mut row_idx = 1;

    worksheet.write_string(0, 0, "姓名").unwrap();
    worksheet.write_string(0, 1, "金额(元)").unwrap();

    for score in month_score {
        worksheet
            .write_string(row_idx as u32, 0, score.student_name)
            .unwrap();
        worksheet.write(row_idx as u32, 1, score.exchanged).unwrap();
        row_idx += 1;
    }
    let file_name = format!(
        "开源实习 临时用工人员劳务费上报表-{}年{}月.xlsx",
        now.year(),
        now.month()
    );
    workbook.save(file_name).unwrap();
    Ok(())
}

async fn calculate_bonus(state: State<AppState>) -> Result<Json<CommonResult<()>>, CommonError> {
    let now = Utc::now();
    let last_month = get_last_month(now.naive_utc().into());

    let month_score = state
        .score_stg()
        .list_score_by_month(now.year(), now.month() as i32)
        .await
        .unwrap();
    let students: Vec<String> = month_score.iter().map(|x| x.github_login.clone()).collect();

    state.score_stg().calculate_bonus(month_score).await;
    let last_month_score = state
        .score_stg()
        .list_score_by_month(last_month.0, last_month.1)
        .await
        .unwrap();
    state
        .score_stg()
        .calculate_unactive_bonus(last_month_score, students)
        .await
        .unwrap();

    Ok(Json(CommonResult::success(None)))
}
