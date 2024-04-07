use rdkafka::producer::FutureRecord;
use rdkafka::{
    config::{ClientConfig, RDKafkaLogLevel},
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::Message,
    producer::FutureProducer,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::router::KafkaMessage;
use crate::secrets::Secrets;

pub struct KafkaProducer {
    producer: FutureProducer,
    pub secrets: Secrets,
}

impl KafkaProducer {
    fn new(secrets: Secrets) -> Self {
        Self {
            producer: create_kafka_producer_upstash(&secrets),
            secrets,
        }
    }

    async fn send_message<T: serde::Serialize>(&self, message: T, topic: &str) {
        let message = serde_json::to_string(&message).unwrap().into_bytes();
        let rec = FutureRecord::to(topic)
            .key("")
            .payload(&message)
            .timestamp(now());

        let _res = self.producer.send_result(rec).unwrap().await.unwrap();
    }
}

pub fn create_kafka_producer_upstash(secrets: &Secrets) -> FutureProducer {
    let log_level: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", secrets.url())
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.username", secrets.username())
        .set("sasl.password", secrets.password())
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    log_level
}

pub fn create_kafka_consumer_upstash(secrets: &Secrets) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", secrets.url())
        .set("sasl.mechanism", "SCRAM-SHA-256")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.username", secrets.username())
        .set("sasl.password", secrets.password())
        .set("group.id", "hello")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed")
}

pub async fn run_consumer_upstash(secrets: &Secrets, target_topic: &str) {
    let prod = create_kafka_producer_upstash(secrets);
    let con = create_kafka_consumer_upstash(secrets);

    con.subscribe(&["sales_orders"])
        .expect("Failed to subscribe to topics");

    loop {
        match con.recv().await {
            Err(e) => println!("kafka error: {e}"),
            Ok(message) => {
                let payload = message
                    .payload_view::<str>()
                    .expect("Error unwrapping!")
                    .expect("No payload :(");

                println!("Payload received: {payload}");

                con.commit_message(&message, CommitMode::Async).unwrap();

                let rec = FutureRecord::to(target_topic)
                    .key("")
                    .payload("Hello world back to you too!")
                    .timestamp(now());

                let _res = prod.send_result(rec).unwrap().await.unwrap();
            }
        }
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}

pub fn create_kafka_producer() -> FutureProducer {
    let url = std::env::var("KAFKA_URL").unwrap();

    let log_level: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", url)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    log_level
}

pub fn create_kafka_consumer() -> StreamConsumer {
    let url = std::env::var("KAFKA_URL").unwrap();

    ClientConfig::new()
        .set("group.id", "test")
        .set("bootstrap.servers", url)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed")
}

pub async fn run_consumer(target_topic: &str) {
    let prod = create_kafka_producer();
    let con = create_kafka_consumer();

    con.subscribe(&["messages"])
        .expect("Failed to subscribe to topics");

    loop {
        match con.recv().await {
            Err(e) => println!("kafka error: {e}"),
            Ok(message) => {
                let payload = message.payload().unwrap();

                let payload: KafkaMessage = serde_json::from_slice(payload).unwrap();
                println!("Payload received: {payload:?}");

                con.commit_message(&message, CommitMode::Async).unwrap();
            }
        }
    }
}
