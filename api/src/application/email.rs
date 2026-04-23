use anyhow::Result;
use axum::extract::State;

use crate::{AppState, email::EmailSender};

pub async fn announcement(state: State<AppState>, template_id: &str) -> Result<()> {
    EmailSender::notice_all_email(state, template_id).await?;
    Ok(())
}
