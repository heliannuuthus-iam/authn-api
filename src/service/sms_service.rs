use anyhow::Context;
use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, AsyncSmtpTransport},
    AsyncTransport, Message, Tokio1Executor,
};

use crate::{
    common::{client::REQWEST, config::env_var, errors::Result, nacos::NACOS_CLIENT},
    dto::sms::{SmsConfig, SmsContext},
};

const FORUM_SERVER: &str = "forum-server";
const FORUM_SERVER_CLUSTER: &str = "default";

async fn get_sms_config(sms_temp_id: i64) -> Result<SmsConfig> {
    let server = NACOS_CLIENT
        .select_one_healthy_instance(
            FORUM_SERVER.to_string(),
            Option::None,
            vec![FORUM_SERVER_CLUSTER.to_string()],
            true,
        )
        .await
        .context("nacos instant get failed")?;
    Ok(REQWEST
        .get(format!(
            "http://{}:{}/smsconfig/{}",
            server.ip, server.port, sms_temp_id
        ))
        .send()
        .await
        .context("get sms config failed")?
        .json::<SmsConfig>()
        .await
        .context("sms config deserialize failed")?)
}

pub async fn send_msg(id: i64) -> Result<()> {
    let mut context = get_sms_config(id).await.map(SmsContext::from)?;
    let message = context
        .render()
        .context(format!("sms template reader error: {}", id))?;
    // Open a remote connection to gmail
    AsyncSmtpTransport::<Tokio1Executor>::relay(env_var::<String>("SMTP_SERVER").as_str())
        .context("relay smtp server failed")?
        .credentials(Credentials::new(
            env_var::<String>("SMTP_SENDER"),
            env_var::<String>("SMTP_CREDENTIALS"),
        ))
        .build()
        .send(
            Message::builder()
                .from(env_var::<String>("SMTP_SENDER").parse().unwrap())
                .to(context.receiver.parse().unwrap())
                .subject(context.subject)
                .header(ContentType::TEXT_HTML)
                .body(message)
                .context("message build failed")?,
        )
        .await
        .context(format!("send email failed: {}", id))?;
    Ok(())
}
