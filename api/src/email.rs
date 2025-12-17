use std::env;

use lettre::message::{SinglePart, header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tera::{Context, Tera};

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
        let tera = Tera::new("templates/*").unwrap();
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

        let creds = Credentials::new(
            env::var("POSTMARK_AK").unwrap(),
            env::var("POSTMARK_SK").unwrap(),
        );

        let mailer = SmtpTransport::starttls_relay("smtp.postmarkapp.com")
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => println!("邮件发送成功"),
            Err(e) => eprintln!("邮件发送失败: {:?}", e),
        }
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
            "monthly_points_summary.html",
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
