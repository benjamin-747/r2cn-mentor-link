use std::io::Cursor;

use anyhow::Result;
use axum::extract::State;
use chrono::{Datelike, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rust_xlsxwriter::Workbook;
use sea_orm::{Set, TryIntoModel};

use common::date::get_last_month;
use entity::monthly_score;
use service::model::score::{CommonScore, ScoreDto, load_score_strategy};

use crate::{AppState, email::EmailSender, model::score::ExportExcel};

pub struct ScoreExportResult {
    pub file_data: Vec<u8>,
    pub content_disposition: String,
}

pub async fn export_excel_data(
    state: &AppState,
    params: &ExportExcel,
) -> Result<ScoreExportResult> {
    let monthly_records = state
        .score_stg()
        .list_score_by_month(params.year, params.month)
        .await?;
    let finished_tasks = state
        .task_stg()
        .search_finished_task_with_date(params.year, params.month)
        .await?;

    let mut workbook = Workbook::new();
    let sheet1 = workbook.add_worksheet().set_name("当月积分总计")?;
    for col in 0..6 {
        sheet1.set_column_width(col, 18)?;
    }
    sheet1.write_string(0, 0, "姓名")?;
    sheet1.write_string(0, 1, "Student ID")?;
    sheet1.write_string(0, 2, "上个月结转分数")?;
    sheet1.write_string(0, 3, "本月新增分数")?;
    sheet1.write_string(0, 4, "本月转换分数")?;
    sheet1.write_string(0, 5, "金额(元)")?;

    let mut score_row_idx = 1;
    for score in monthly_records {
        if score.exchanged != 0 {
            sheet1.write_string(score_row_idx as u32, 0, score.student_name)?;
            sheet1.write_string(score_row_idx as u32, 1, score.student_id)?;
            sheet1.write_number(score_row_idx as u32, 2, score.carryover_score)?;
            sheet1.write_number(score_row_idx as u32, 3, score.new_score)?;
            sheet1.write_number(score_row_idx as u32, 4, score.consumption_score)?;
            sheet1.write_number(score_row_idx as u32, 5, score.exchanged)?;
            score_row_idx += 1;
        }
    }

    let sheet2 = workbook.add_worksheet().set_name("当月任务详情")?;
    for col in 0..5 {
        sheet2.set_column_width(col, 18)?;
    }
    sheet2.write_string(0, 0, "学生ID")?;
    sheet2.write_string(0, 1, "导师ID")?;
    sheet2.write_string(0, 2, "任务标题")?;
    sheet2.write_string(0, 3, "任务链接")?;
    sheet2.write_string(0, 4, "任务分数")?;

    for (task_row_idx, task) in finished_tasks.into_iter().enumerate() {
        let row = (task_row_idx + 1) as u32;
        sheet2.write_string(row, 0, task.student_id.unwrap_or_default())?;
        sheet2.write_string(row, 1, task.mentor_login)?;
        sheet2.write(row, 2, task.issue_title)?;
        sheet2.write(row, 3, task.issue_link)?;
        sheet2.write_number(row, 4, task.score)?;
    }

    let file_name = format!(
        "开源实习-人员劳务费统计表-{}年{}月.xlsx",
        params.year, params.month
    );
    let encoded_filename = utf8_percent_encode(&file_name, NON_ALPHANUMERIC).to_string();
    let disposition = format!("attachment; filename*=UTF-8''{}", encoded_filename);
    let mut buffer = Cursor::new(Vec::new());
    workbook.save_to_writer(&mut buffer)?;

    Ok(ScoreExportResult {
        file_data: buffer.into_inner(),
        content_disposition: disposition,
    })
}

pub async fn calculate_bonus(state: &AppState) -> Result<()> {
    let now = Utc::now().naive_utc();
    let calculate_month = get_last_month(now.into());
    let monthly_records = state
        .score_stg()
        .list_score_by_month(calculate_month.year(), calculate_month.month() as i32)
        .await?;

    for model in &monthly_records {
        let sum = model.carryover_score + model.new_score;
        let student = state
            .student_stg()
            .get_student_by_student_id(&model.student_id)
            .await?;
        let consume_score = {
            let strategy = if let Some(student) = &student {
                load_score_strategy(student.contract_end_date, calculate_month)
            } else {
                tracing::error!("Invalid Student Status:{}", model.student_id);
                Box::new(CommonScore)
            };
            strategy.consumed_score(sum)
        };
        let mut a_model: monthly_score::ActiveModel = model.clone().into();
        a_model.consumption_score = Set(consume_score);
        a_model.exchanged = Set(consume_score * 50);
        a_model.update_at = Set(Utc::now().naive_utc());
        state.score_stg().update_score(a_model.clone()).await?;
        let last_month: ScoreDto = a_model.try_into_model()?.into();
        state
            .score_stg()
            .insert_or_update_carryover_score(last_month)
            .await?;
    }
    Ok(())
}

pub async fn send_monthly_email(state: State<AppState>) -> Result<()> {
    let now = Utc::now().naive_utc();
    let calculate_month = get_last_month(now.into());
    let monthly_records = state
        .score_stg()
        .list_score_by_month(calculate_month.year(), calculate_month.month() as i32)
        .await?;

    for record in monthly_records {
        let student = state
            .student_stg()
            .get_student_by_student_id(&record.student_id)
            .await
            .unwrap_or(None);
        let last_month: ScoreDto = record.into();
        let state_clone = state.clone();
        if last_month.consumption_score != 0 {
            tokio::spawn(async move {
                EmailSender::monthly_score_email(state_clone, student, last_month).await;
            });
        }
    }
    Ok(())
}
