use lettre::{
    transport::smtp::{
        authentication::Credentials, response::Response as LettreResponse, Error as LettreError,
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::env::var;

lazy_static::lazy_static! {
    pub static ref EMAIL_USERNAME: String = var("EMAIL_USERNAME").expect("email username must be set for password reset responses");
    static ref EMAIL_PASSWORD: String = var("EMAIL_PASSWORD").expect("email access password must be set for password reset responses");
    static ref CREDS: Credentials = Credentials::new(EMAIL_USERNAME.to_string(), EMAIL_PASSWORD.to_string());
}

pub async fn sanity_check() {
    let name = format!("sanity check <{}>", EMAIL_USERNAME.to_string());
    let email = Message::builder()
        .from(name.parse().unwrap())
        .to(name.parse().unwrap())
        .subject("Ensuring provided email is valid")
        .body("SANITY CHECK".to_string())
        .unwrap();

    match send(email).await {
        Ok(_) => {}
        Err(e) => panic!("email sanity check failed with {:?}", e),
    }
}

pub async fn send(msg: Message) -> Result<LettreResponse, LettreError> {
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(CREDS.clone())
            .build();

    mailer.send(msg).await
}
