use std::io::Cursor;

use axum::{
    Json, Router,
    body::Body,
    extract::{Query, State},
    response::Response,
    routing::{get, post},
};
use chrono::{Datelike, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rust_xlsxwriter::Workbook;
use sea_orm::{Set, TryIntoModel};

use common::{date::get_last_month, errors::CommonError, model::CommonResult};
use entity::monthly_score;
use service::model::score::{CommonScore, ScoreDto, load_score_strategy};

use crate::{AppState, email::EmailSender, model::score::ExportExcel};

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

    let sheet1 = workbook.add_worksheet().set_name("当月积分总计").unwrap();

    for col in 0..6 {
        sheet1.set_column_width(col, 18).unwrap();
    }

    let mut row_idx = 1;

    sheet1.write_string(0, 0, "姓名").unwrap();
    sheet1.write_string(0, 1, "GitHub ID").unwrap();
    sheet1.write_string(0, 2, "上个月结转分数").unwrap();
    sheet1.write_string(0, 3, "本月新增分数").unwrap();
    sheet1.write_string(0, 4, "本月转换分数").unwrap();
    sheet1.write_string(0, 5, "金额(元)").unwrap();

    for score in monthly_records {
        if score.exchanged != 0 {
            sheet1
                .write_string(row_idx as u32, 0, score.student_name)
                .unwrap();
            sheet1
                .write_string(row_idx as u32, 1, score.github_login)
                .unwrap();
            sheet1
                .write_number(row_idx as u32, 2, score.carryover_score)
                .unwrap();
            sheet1
                .write_number(row_idx as u32, 3, score.new_score)
                .unwrap();
            sheet1
                .write_number(row_idx as u32, 4, score.consumption_score)
                .unwrap();
            sheet1
                .write_number(row_idx as u32, 5, score.exchanged)
                .unwrap();
            row_idx += 1;
        }
    }

    let sheet2 = workbook.add_worksheet().set_name("当月任务详情").unwrap();

    for col in 0..5 {
        sheet2.set_column_width(col, 18).unwrap();
    }

    sheet2.write_string(0, 0, "学生GitHub ID").unwrap();
    sheet2.write_string(0, 1, "导师GitHub ID").unwrap();
    sheet2.write_string(0, 2, "任务标题").unwrap();
    sheet2.write_string(0, 3, "任务链接").unwrap();
    sheet2.write_string(0, 4, "任务分数").unwrap();

    let mut row_idx = 1;

    let finished_tasks = state
        .task_stg()
        .search_finished_task_with_date(params.year, params.month)
        .await
        .unwrap();

    for task in finished_tasks {
        sheet2
            .write_string(
                row_idx as u32,
                0,
                task.student_github_login.unwrap_or_default(),
            )
            .unwrap();
        sheet2
            .write_string(row_idx as u32, 1, task.mentor_github_login)
            .unwrap();
        sheet2
            .write(row_idx as u32, 2, task.github_issue_title)
            .unwrap();
        sheet2
            .write(row_idx as u32, 3, task.github_issue_link)
            .unwrap();
        sheet2.write_number(row_idx as u32, 4, task.score).unwrap();
        row_idx += 1;
    }

    let file_name = format!(
        "开源实习-人员劳务费统计表-{}年{}月.xlsx",
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
            let strategy = if let Some(student) = &student {
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
            .insert_or_update_carryover_score(last_month.clone())
            .await
            .unwrap();
        if last_month.new_score != 0 {
            let state_clone = state.clone();
            tokio::spawn(async move {
                EmailSender::monthly_score_email(state_clone, student, last_month).await
            });
        }
    }
    Ok(Json(CommonResult::success(None)))
}
