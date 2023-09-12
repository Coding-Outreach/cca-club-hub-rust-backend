use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::Credentials, response::Response as LettreResponse, Error as LettreError,
    },
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::env::var;

lazy_static::lazy_static! {
    static ref EMAIL_USERNAME: String = var("EMAIL_USERNAME").expect("email username must be set for password reset responses");
    static ref EMAIL_PASSWORD: String = var("EMAIL_PASSWORD").expect("email access password must be set for password reset responses");
    static ref CREDS: Credentials = Credentials::new(EMAIL_USERNAME.to_string(), EMAIL_PASSWORD.to_string());
    pub static ref EMAIL_ADDRESS: Address = EMAIL_USERNAME.to_string().parse::<Address>().expect("invalid email username");
    // maybe figure out a better place to the HOST var
    pub static ref FRONTEND_HOST: String = var("FRONTEND_HOST").expect("FRONTEND_HOST must be set for correct password reset urls to be generated");
}

pub async fn sanity_check() -> Result<(), LettreError> {
    let mbox = Mailbox::new(Some("apathetic programmers".to_string()), EMAIL_ADDRESS.clone());
    let email = Message::builder()
        .to(mbox.clone())
        .from(mbox)
        .subject("Ensuring provided email is valid")
        .body("SANITY CHECK".to_string())
        .unwrap();

    send(email).await?;

    Ok(())
}

pub async fn send(msg: Message) -> Result<LettreResponse, LettreError> {
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(CREDS.clone())
            .build();

    mailer.send(msg).await
}
