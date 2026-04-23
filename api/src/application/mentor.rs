use anyhow::Result;
use service::storage::mentor_stg::MentorRes;

use crate::{AppState, model::mentor::UpdateMentorStatusRequest};

pub async fn new_mentor(
    state: &AppState,
    payload: crate::model::mentor::NewMentor,
) -> Result<MentorRes> {
    let active_model = payload.into();
    let model = state.mentor_stg().new_mentor(active_model).await?;
    Ok(model.into())
}

pub async fn change_mentor_status(
    state: &AppState,
    payload: UpdateMentorStatusRequest,
) -> Result<MentorRes> {
    let model = state
        .mentor_stg()
        .change_mentor_status(&payload.login, payload.status)
        .await?;
    Ok(model.into())
}
