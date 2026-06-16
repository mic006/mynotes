use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::config::AppConfigMailAlert;

/// Sends an email with the provided HTML body using the configuration.
pub async fn send_mail(
    body_html: &str,
    config: &AppConfigMailAlert,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the email message
    let email = Message::builder()
        .from(config.sender_email.parse()?)
        // Using the configured SMTP user as the recipient for notifications.
        .to(config.smtp_user.parse()?)
        .subject(&config.mail_title)
        .singlepart(SinglePart::html(body_html.to_string()))?;

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

    // Determine security based on port: 465 uses Implicit TLS (Wrapper), 587 uses STARTTLS (Required).
    let tls_parameters = TlsParameters::new(config.smtp_addr.clone())?;
    let tls_mode = if config.smtp_port == 465 {
        Tls::Wrapper(tls_parameters)
    } else {
        Tls::Required(tls_parameters)
    };

    // Setup the SMTP transport
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_addr)?
        .port(config.smtp_port)
        .tls(tls_mode)
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(email).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "to be launched manually, after providing a suitable configuration"]
    #[tokio::test]
    async fn test_send_mail() -> Result<(), Box<dyn std::error::Error>> {
        let config = AppConfigMailAlert {
            mail_title: "UT mynotes".to_string(),
            smtp_addr: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_user: "user".to_string(),
            smtp_password: "password".to_string(),
            sender_email: "user@example.com".to_string(),
            ..Default::default()
        };

        send_mail(
            "<h1>Unit test</h1><p>send_mail() is working fine</p>",
            &config,
        )
        .await
    }
}
