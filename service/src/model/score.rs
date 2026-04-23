use chrono::NaiveDate;
use entity::monthly_score;
use serde::{Deserialize, Serialize};

pub trait ScoreStrategy {
    fn consumed_score(&self, score: i32) -> i32;
}

/// 通用计算逻辑，大于40分按阶段发放
pub struct CommonScore;
impl ScoreStrategy for CommonScore {
    fn consumed_score(&self, score: i32) -> i32 {
        if score >= 100 {
            return 100;
        } else if score >= 80 {
            return 80;
        } else if score >= 60 {
            return 60;
        } else if score >= 40 {
            return 40;
        }
        0
    }
}

/// 截止日期规则，达到截止日期后按每月最多100分发放，不足100按实际发放
pub struct DeadlineScore;
impl ScoreStrategy for DeadlineScore {
    fn consumed_score(&self, score: i32) -> i32 {
        if score >= 100 {
            return 100;
        }
        // 合同截止，小于100分全部发放
        score
    }
}

/// 活动结束规则：一次性发放全部剩余积分
pub struct ActivityEndScore;
impl ScoreStrategy for ActivityEndScore {
    fn consumed_score(&self, score: i32) -> i32 {
        score
    }
}

pub fn load_score_strategy(
    contract_end_date: Option<NaiveDate>,
    date: NaiveDate,
) -> Box<dyn ScoreStrategy> {
    let activity_end_month = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid activity end month");
    if date >= activity_end_month {
        return Box::new(ActivityEndScore);
    }

    // 此处计算的时候抹去了合同的日期，只计算到月份，日期默认为1号
    if let Some(contract_end_date) = contract_end_date
        && contract_end_date <= date
    {
        return Box::new(DeadlineScore);
    }
    Box::new(CommonScore)
}

#[derive(PartialEq, Eq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreDto {
    pub id: i32,
    pub student_id: String,
    pub student_name: String,
    pub year: i32,
    pub month: i32,
    pub carryover_score: i32,
    pub new_score: i32,
    pub consumption_score: i32,
    pub exchanged: i32,
}

impl ScoreDto {
    pub fn score_balance(&self) -> i32 {
        self.carryover_score + self.new_score - self.consumption_score
    }

    pub fn score_total(&self) -> i32 {
        self.carryover_score + self.new_score
    }
}

impl From<monthly_score::Model> for ScoreDto {
    fn from(value: monthly_score::Model) -> Self {
        Self {
            id: value.id,
            student_id: value.student_id,
            student_name: value.student_name,
            year: value.year,
            month: value.month,
            carryover_score: value.carryover_score,
            new_score: value.new_score,
            consumption_score: value.consumption_score,
            exchanged: value.exchanged,
        }
    }
}
