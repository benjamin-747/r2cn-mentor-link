use std::{env, fs};
use std::path::{Path, PathBuf};

use axum::extract::State;
use entity::{student, task};
use lettre::message::{header, Attachment, Body, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use service::model::score::ScoreDto;
use tera::{Context, Tera};

use crate::AppState;

// 用 mrml 将 MJML 转换为 HTML
pub fn render_mjml(template_name: &str, context: &Context) -> Result<String, String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("templates/mjml/*");
    let tera = Tera::new(path.to_str().unwrap()).map_err(|e| format!("Tera 初始化失败: {}", e))?;
    let mjml_content = tera
        .render(template_name, context)
        .map_err(|e| format!("Tera 渲染失败: {}", e))?;

    let mjml_content = mjml_content.replace("\r\n", "\n");

    let root = mrml::parse(&mjml_content).map_err(|e| format!("MJML 解析失败: {}", e))?;
    let opts = mrml::prelude::render::RenderOptions::default();
    let html = root
        .render(&opts)
        .map_err(|e| format!("MJML 渲染失败: {}", e))?;
    Ok(html)
}

/// 创建带有 Content-ID 的内嵌图片附件
pub fn create_cid_attachment(image_path: &str, cid: &str) -> Result<SinglePart, String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(image_path);
    let image_data = fs::read(Path::new(&path)).map_err(|e| format!("读取图片失败: {}", e))?;

    let body = Body::new(image_data);
    let content_type_header: header::ContentType = "image/png".parse().unwrap();
    Ok(Attachment::new_inline(cid.to_string()).body(body, content_type_header))
}

/// 根据模板名称获取需要内嵌的图片
pub fn cid_images_for_template(template_name: &str) -> Vec<(&'static str, &'static str)> {
    let mut imgs: Vec<(&'static str, &'static str)> =
        vec![("templates/image/background.png", "background")];

    match template_name {
        "task_assigned.mjml" => {
            imgs.push(("templates/image/task_assigned.png", "task_status"));
        }
        "task_failed.mjml" => {
            imgs.push(("templates/image/task_failed.png", "task_status"));
        }
        "task_completed_points.mjml" => {
            imgs.push(("templates/image/task_completed.png", "task_status"));
        }
        "monthly_points_summary.mjml" => {
            imgs.push(("templates/image/task_points.png", "task_status"));
        }
        _ => {}
    }

    imgs
}

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
        let render_result = if self.template_name.ends_with(".mjml") {
            render_mjml(&self.template_name, &self.context)
        } else {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("templates/*");
            Tera::new(path.to_str().unwrap())
                .map_err(|e| format!("Tera 初始化失败: {}", e))
                .and_then(|t| t.render(&self.template_name, &self.context).map_err(|e| format!("Tera 渲染失败: {}", e)))
        };

        let html_body = match render_result {
            Ok(body) => body,
            Err(e) => {
                tracing::error!("邮件模板渲染失败: {}", e);
                return;
            }
        };

        let html_part = SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(html_body);

        let mut email_builder = Message::builder()
            .from("no-reply@r2cn.dev".parse().unwrap())
            .to(self.receiver.parse().unwrap())
            .subject(self.subject.clone());

        let email = if self.template_name.ends_with(".mjml") {
            let mut multipart = MultiPart::related().singlepart(html_part);
            for (img_path, cid) in cid_images_for_template(&self.template_name) {
                match create_cid_attachment(img_path, cid) {
                    Ok(part) => {
                        multipart = multipart.singlepart(part);
                    }
                    Err(err) => tracing::warn!("内嵌图片加载失败 {}: {}", img_path, err),
                }
            }
            email_builder.multipart(multipart)
        } else {
            email_builder.singlepart(html_part)
        }
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

                let sender =
                    EmailSender::new(
                        "task_failed.mjml",
                         "R2CN任务失败通知/R2CN Task Failure",
                         email_context,
                         &student.email
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
                "task_assigned.mjml",
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
                    "task_completed_points.mjml",
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
                "monthly_points_summary.mjml",
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

    use super::{cid_images_for_template, create_cid_attachment, render_mjml, EmailSender};
    use lettre::{
        Message, SmtpTransport, Transport,
        message::{header, MultiPart, SinglePart},
        transport::smtp::authentication::Credentials,
    };

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
            "task_assigned.mjml",
            "R2CN任务完成",
            email_context,
            "yetianxing2014@gmail.com",
        );

        let html_body = render_mjml(&sender.template_name, &sender.context).unwrap();
        let mut multipart = MultiPart::related().singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html_body),
        );
        for (img_path, cid) in cid_images_for_template(&sender.template_name) {
            let part = create_cid_attachment(img_path, cid).unwrap();
            multipart = multipart.singlepart(part);
        }
        let email = Message::builder()
             .from("no-reply@r2cn.dev".parse().unwrap())
            .to(sender.receiver.parse().unwrap())
            .subject(sender.subject.clone())
            .multipart(multipart)
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
