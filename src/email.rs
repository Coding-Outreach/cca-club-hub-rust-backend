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
    pub static ref EMAIL_ADDRESS: Address = EMAIL_USERNAME.parse::<Address>().expect("invalid email username");
    // maybe figure out a better place to the HOST var
    pub static ref HOST: String = var("HOST").expect("HOST must be set for correct password reset urls to be generated");
}

pub async fn sanity_check() {
    let mbox = Mailbox::new(None, EMAIL_ADDRESS.clone());
    let email = Message::builder()
        .to(mbox)
        .subject("Ensuring provided email is valid")
        .body("SANITY CHECK".to_string())
        .unwrap();

    send(email).await.expect("email sanity check failed");
}

pub async fn send(msg: Message) -> Result<LettreResponse, LettreError> {
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(CREDS.clone())
            .build();

    mailer.send(msg).await
}
