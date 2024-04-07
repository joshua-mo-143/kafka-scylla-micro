use rdkafka::producer::FutureProducer;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use tokio::net::TcpListener;

mod kafka;
mod queries;
mod router;
mod secrets;

use queries::{KEYSPACE, MESSAGES_TABLE};

#[derive(Clone)]
pub struct AppState {
    kafka_p: FutureProducer,
    scylla: Arc<Session>,
}

#[tokio::main]
async fn main() {
    let scylla = SessionBuilder::new()
        .known_node("localhost:9042")
        .build()
        .await
        .unwrap();

    let prepared = scylla.prepare(KEYSPACE).await.unwrap();

    scylla.execute(&prepared, []).await.unwrap();

    let prepared = scylla.prepare(MESSAGES_TABLE).await.unwrap();

    scylla.execute(&prepared, []).await.unwrap();

    scylla.use_keyspace("tutorial", false).await.unwrap();

    tokio::spawn( async move {
        check_cdc().await;
    });

    tokio::spawn(async {
        kafka::run_consumer("sales_orders").await;
    });

    let state = AppState {
        kafka_p: kafka::create_kafka_producer(),
        scylla: Arc::new(scylla),
    };

    let router = router::init_router(state);

    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn check_cdc() {
    let scylla = SessionBuilder::new()
        .use_keyspace("tutorial", false)
        .known_node("localhost:9042")
        .build()
        .await
        .unwrap();

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    loop {
        let rows_opt = scylla.query("SELECT message_id, name, message from messages_scylla_cdc_log", &[]).await.unwrap().rows;

        if let Some(rows) = rows_opt {
            println!("{rows:?}");
        }

       interval.tick().await; 
    }
}
