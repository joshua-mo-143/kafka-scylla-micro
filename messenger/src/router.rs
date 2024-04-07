use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

async fn hello_world(State(state): State<AppState>) -> &'static str {
    "Hello world!"
}

pub fn init_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/messages", get(list_messages).post(create_message))
        .route(
            "/messages/:id",
            get(find_message).put(update_message).delete(delete_message),
        )
        .with_state(state)
}

#[derive(scylla::FromRow, Debug, Serialize, Deserialize)]
struct Message {
    message_id: Uuid,
    name: String,
    message: String,
}

#[derive(Deserialize, scylla::SerializeRow)]
struct NewMessage {
    name: String,
    message: String,
}

async fn list_messages(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let rows_opt = state
        .scylla
        .query("SELECT message_id, name, message FROM messages", &[])
        .await
        .unwrap()
        .rows;

    let Some(rows) = rows_opt else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut vec: Vec<Message> = Vec::new();

    for row in rows {
        vec.push(row.into_typed::<Message>().unwrap());
    }

    Ok(Json(vec))
}

async fn find_message(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let rows_opt = state
        .scylla
        .query(
            "SELECT message_id, name, message FROM messages WHERE message_id = ?",
            (id,),
        )
        .await
        .unwrap()
        .rows;

    let Some(rows) = rows_opt else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let mut vec: Vec<Message> = Vec::new();

    for row in rows {
        vec.push(row.into_typed::<Message>().unwrap());
    }

    Ok(Json(vec))
}

async fn create_message(
    State(state): State<AppState>,
    Json(json): Json<NewMessage>,
) -> impl IntoResponse {
    let statement = state
        .scylla
        .prepare("INSERT into messages (message_id, name, message) VALUES (uuid(), ?, ?)")
        .await
        .unwrap();

    state.scylla.execute(&statement, json).await.unwrap();

    let message = KafkaMessage {
        action: Action::Create,
        data: None,
    };

    send_message(state.kafka_p, message).await;

    StatusCode::OK
}

async fn update_message(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(json): Json<NewMessage>,
) -> impl IntoResponse {
    let statement = state
        .scylla
        .prepare("update messages set name = ?, message = ? where message_id = ?")
        .await
        .unwrap();

    state
        .scylla
        .execute(&statement, (json.name, json.message, id))
        .await
        .unwrap();

    let message = KafkaMessage {
        action: Action::Update,
        data: None,
    };

    send_message(state.kafka_p, message).await;

    StatusCode::OK
}

async fn delete_message(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let statement = state
        .scylla
        .prepare("DELETE FROM messages where message_id = ?")
        .await
        .unwrap();

    state.scylla.execute(&statement, (id,)).await.unwrap();

    let message = KafkaMessage {
        action: Action::Delete,
        data: None,
    };

    send_message(state.kafka_p, message).await;

    StatusCode::OK
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    action: Action,
    data: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Action {
    Create,
    Update,
    Delete,
}

async fn send_message(producer: FutureProducer, message: KafkaMessage) {
    let json_message = serde_json::to_string(&message).unwrap();
    let record: FutureRecord<str, String> =
        FutureRecord::to("messages").payload(&json_message).key("1");

    producer
        .send_result(record)
        .unwrap()
        .await
        .unwrap()
        .unwrap();
}
