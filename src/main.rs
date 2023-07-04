use backend::configuration::get_configuration;
use backend::startup::run;
use backend::telemetry::{add_file_sink, create_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() {
    let subscriber = create_subscriber("{{project-name}}".into(), "info".into(), std::io::stdout);
    let subscriber = add_file_sink(subscriber, "{{project-name}}".into());
    init_subscriber(subscriber);

    let listener = TcpListener::bind("127.0.0.1:9000").expect("Failed to bind TcpListener");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        PgPool::connect(configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    run(listener, connection_pool).await.unwrap();
}
