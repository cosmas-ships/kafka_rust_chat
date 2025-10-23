use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message as KafkaMessage,
};

use std::time::Duration;
use tokio::{sync::broadcast, task::JoinHandle};
use uuid::Uuid;

use std::{collections::HashSet, sync::{Arc, Mutex}};


#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel::<String>(100);

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation failed");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", Uuid::new_v4().to_string())
        .set("auto.offset.reset", "latest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["chat-room"])
        .expect("Can't subscribe to chat-room");

    // Track processed message IDs to avoid duplicates
    let seen_messages = Arc::new(Mutex::new(HashSet::<String>::new()));

    let tx_clone = tx.clone();
    let seen_clone = seen_messages.clone();
    tokio::spawn(async move {
        let mut stream = consumer.stream();
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                if let Some(Ok(text)) = msg.payload_view::<str>() {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(id) = value["id"].as_str() {
                            let mut seen = seen_clone.lock().unwrap();
                            if !seen.contains(id) {
                                seen.insert(id.to_string());
                                println!("Kafka → {}", text);
                                let _ = tx_clone.send(text.to_string());
                            }
                        }
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/ws", get(handle_ws))
        .with_state((tx.clone(), producer, seen_messages));

    println!("WebSocket server running at ws://localhost:3001/ws");

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:3001")
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn handle_ws(
    ws: WebSocketUpgrade,
    State((tx, producer, seen_messages)): State<(broadcast::Sender<String>, FutureProducer, Arc<Mutex<HashSet<String>>>)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx, producer, seen_messages))
}

pub async fn handle_socket(
    socket: WebSocket,
    tx: broadcast::Sender<String>,
    producer: FutureProducer,
    seen_messages: Arc<Mutex<HashSet<String>>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let send_task: JoinHandle<()> = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx_clone = tx.clone();
    let seen_clone = seen_messages.clone();
    let recv_task: JoinHandle<()> = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&text) {
                let id = Uuid::new_v4().to_string();
                value["id"] = serde_json::Value::String(id.clone());
                value["timestamp"] = serde_json::Value::String(Utc::now().to_rfc3339());
                let payload = serde_json::to_string(&value).unwrap();

                // Mark as seen before sending to Kafka
                seen_clone.lock().unwrap().insert(id.clone());

                // Send to Kafka
                let _ = producer
                    .send(
                        FutureRecord::to("chat-room").payload(&payload).key("chat"),
                        Duration::from_secs(0),
                    )
                    .await;

                // Broadcast locally immediately
                let _ = tx_clone.send(payload);
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
