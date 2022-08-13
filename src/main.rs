use std::env;
use std::str::FromStr;
use aws_lambda_events::event::cognito::{CognitoEventUserPoolsPostConfirmation};
use lambda_http::{lambda_runtime,
                  lambda_runtime::Error, service_fn};
use lambda_runtime::LambdaEvent;
use serde_json::{json, Value};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(start_lambda)).await?;

    Ok(())
}

async fn start_lambda(event: LambdaEvent<CognitoEventUserPoolsPostConfirmation>) -> Result<Value, Error> {
    let customer_id = Uuid::from_str(event.payload.request.user_attributes.get("sub").unwrap())?;
    let customer_email = event.payload.request.user_attributes.get("email").unwrap();
    println!("Registration sync lambda started up for user with id: {}, and email: {}", &customer_id, customer_email);

    let mut database_connection = create_connection().await;

    println!("Connection is established");

    sqlx::query("INSERT INTO customer (id, email, created_at) VALUES ($1, $2, 'now()');")
        .bind(customer_id)
        .bind(customer_email)
        .execute(&mut database_connection).await?;

    database_connection.close();

    println!("Database save was success, connection was closed");
    Ok(json!(event.payload))
}

pub async fn create_connection() -> PgConnection {
    let conn = PgConnection::connect(format!("postgres://{}:{}@{}",
                                             env::var("DATABASE_USERNAME")
                                                 .unwrap(),
                                             env::var("DATABASE_PASSWORD")
                                                 .unwrap(),
                                             env::var("DATABASE_URL")
                                                 .unwrap()).as_str()).await;

    conn.unwrap()
}
