use std::env;
use std::path::PathBuf;

use axum::extract::State;
use entity::{student, task};
use lettre::message::{SinglePart, header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use service::model::score::ScoreDto;
use tera::{Context, Tera};

use crate::AppState;

pub struct EmailSender {
    template_name: String,
    subject: String,
    context: Context,
    receiver: String,
}

impl EmailSender {
    pub fn new(template_name: &str, subject: &str, context: Context, receiver: &str) -> Self {
        EmailSender {
            template_name: template_name.to_owned(),
            subject: subject.to_owned(),
            context,
            receiver: receiver.to_owned(),
        }
    }

    pub fn send(&self) {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("templates/*");
        let tera = Tera::new(path.to_str().unwrap()).unwrap();
        let html_body = tera.render(&self.template_name, &self.context).unwrap();
        let email = Message::builder()
            .from("no-reply@r2cn.dev".parse().unwrap())
            .to(self.receiver.parse().unwrap())
            .subject(self.subject.clone())
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html_body),
            )
            .unwrap();

        let creds = Credentials::new(env::var("ZEPTO_AK").unwrap(), env::var("ZEPTO_SK").unwrap());

        let mailer = SmtpTransport::starttls_relay("smtp.zeptomail.com")
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => tracing::info!("邮件发送成功: to {} ", self.receiver),
            Err(e) => tracing::error!("邮件发送失败: {:?}, to {}", e,self.receiver),
        }
    }

    pub async fn failed_email(state: State<AppState>, task: task::Model) {
        if let Some(student_github_login) = &task.student_github_login {
            let student = state
                .student_stg()
                .get_student_by_login(student_github_login)
                .await
                .unwrap();
            if let Some(student) = student {
                let mut email_context = tera::Context::new();
                email_context.insert("student_name", &student.student_name);
                email_context.insert("task_title", &task.github_issue_title);
                email_context.insert("task_link", &task.github_issue_link);
                email_context.insert("mentor_name", &task.mentor_github_login);
                email_context.insert("project_link", &util::project_link(&task));

                let sender = EmailSender::new(
                    "task_failed.html",
                    "R2CN任务失败通知/R2CN Task Failure",
                    email_context,
                    &student.email,
                );
                sender.send();
            }
        }
    }

    pub async fn assigned_email(state: State<AppState>, task: task::Model) {
        if let Some(student_github_login) = &task.student_github_login {
            let student = state
                .student_stg()
                .get_student_by_login(student_github_login)
                .await
                .unwrap()
                .unwrap();

            let mut email_context = tera::Context::new();
            email_context.insert("student_name", &student.student_name);
            email_context.insert("task_title", &task.github_issue_title);
            email_context.insert("task_link", &task.github_issue_link);
            email_context.insert("mentor_name", &task.mentor_github_login);
            email_context.insert("project_link", &util::project_link(&task));
            let sender = EmailSender::new(
                "task_assigned.html",
                "R2CN任务认领通知/R2CN Task Assigned",
                email_context,
                &student.email,
            );
            sender.send();
        }
    }

    pub async fn complete_email(state: State<AppState>, task: task::Model, balance: i32) {
        if let Some(student_github_login) = &task.student_github_login {
            let student = state
                .student_stg()
                .get_student_by_login(student_github_login)
                .await
                .unwrap();
            if let Some(student) = student {
                let mut email_context = tera::Context::new();
                email_context.insert("student_name", &student.student_name);
                email_context.insert("task_title", &task.github_issue_title);
                email_context.insert("task_link", &task.github_issue_link);
                email_context.insert("mentor_name", &task.mentor_github_login);
                email_context.insert("points_total", &balance);
                email_context.insert("project_link", &util::project_link(&task));
                let sender = EmailSender::new(
                    "task_completed_points.html",
                    "R2CN任务完成通知/R2CN Task Successful",
                    email_context,
                    &student.email,
                );
                sender.send();
            }
        }
    }

    pub async fn monthly_score_email(student: Option<student::Model>, last_month: ScoreDto) {
        if let Some(student) = student {
            let mut email_context = tera::Context::new();
            email_context.insert("student_name", &student.student_name);
            email_context.insert("points_earned_month", &last_month.new_score);
            email_context.insert("points_redeemed_month", &last_month.consumption_score);
            email_context.insert("points_balance", &last_month.score_balance());

            let sender = EmailSender::new(
                "monthly_points_summary.html",
                "R2CN月度积分报告/R2CN Monthly Score Report",
                email_context,
                &student.email,
            );
            sender.send();
        }
    }
}

pub mod util {

    use entity::task::{self};

    pub fn project_link(task: &task::Model) -> String {
        format!("https://github.com/{}/{}", task.owner, task.repo)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::EmailSender;
    use lettre::{
        Message, SmtpTransport, Transport,
        message::{
            SinglePart,
            header::{self},
        },
        transport::smtp::authentication::Credentials,
    };
    use tera::Tera;

    #[test]
    pub fn test_email() {
        dotenvy::dotenv().ok();

        let mut email_context = tera::Context::new();
        email_context.insert("student_name", "name");
        email_context.insert("task_title", "title");
        email_context.insert(
            "task_link",
            "https://github.com/benjamin-747/r2cn-bot-test/issues/22",
        );
        email_context.insert(
            "project_link",
            "https://github.com/benjamin-747/r2cn-bot-test",
        );
        email_context.insert("task_id", "123");
        email_context.insert("mentor_name", "name");
        email_context.insert("mentor_email", "email");
        email_context.insert("points_earned", "25");
        email_context.insert("points_total", "100");
        email_context.insert("failure_reason", "fail reason");
        email_context.insert("review_comments", "comments");
        email_context.insert("resubmit_link", "");
        email_context.insert("report_month", "12");
        email_context.insert("generated_at", "20251212");
        email_context.insert("points_earned_month", "60");
        email_context.insert("points_redeemed_month", "100");
        email_context.insert("points_balance", "15");

        let sender = EmailSender::new(
            "task_assigned.html",
            "R2CN任务完成",
            email_context,
            "yetianxing2014@gmail.com",
        );

        let tera = Tera::new("templates/*").unwrap();
        let html_body = tera.render(&sender.template_name, &sender.context).unwrap();
        let email = Message::builder()
            .from("no-reply@r2cn.dev".parse().unwrap())
            .to(sender.receiver.parse().unwrap())
            .subject(sender.subject.clone())
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(html_body),
            )
            .unwrap();

        let creds = Credentials::new(env::var("ZEPTO_AK").unwrap(), env::var("ZEPTO_SK").unwrap());

        let mailer = SmtpTransport::starttls_relay("smtp.zeptomail.com")
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => println!("邮件发送成功"),
            Err(e) => eprintln!("邮件发送失败: {:?}", e),
        }
    }
}
