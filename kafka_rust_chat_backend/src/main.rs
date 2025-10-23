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

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[tokio::main]
async fn main() {
    // Create a broadcast channel for Strings with capacity 100
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Create a Kafka producer connected to local Kafka broker
    // FutureProducer allows async message sending with awaitable results
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation failed");

    // Create a unique Kafka consumer that starts reading from the latest messages
    // Each instance gets a random group ID to avoid offset conflicts
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", Uuid::new_v4().to_string()) // Unique ID for this consumer
        .set("auto.offset.reset", "latest") // Start from newest messages
        .create()
        .expect("Consumer creation failed");

    // Subscribe to the "chat-room" Kafka topic to start receiving messages
    // If subscription fails, the application cannot function properly
    consumer
        .subscribe(&["chat-room"])
        .expect("Can't subscribe to chat-room");

    // Track processed message IDs to avoid duplicates
    let seen_messages = Arc::new(Mutex::new(HashSet::<String>::new()));

    let tx_clone = tx.clone();
    let seen_clone = seen_messages.clone();

    // Background task: Consume Kafka messages, deduplicate, and broadcast
    tokio::spawn(async move {
        // Create a stream of incoming Kafka messages
        let mut stream = consumer.stream();

        // Continuously poll for new messages in an infinite loop
        while let Some(result) = stream.next().await {
            if let Ok(msg) = result {
                if let Some(Ok(text)) = msg.payload_view::<str>() {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(id) = value["id"].as_str() {
                            // Critical section: Check and update seen messages
                            let mut seen = seen_clone.lock().unwrap();
                            if !seen.contains(id) {
                                seen.insert(id.to_string());
                                // println!("Kafka → {}", text);
                                let _ = tx_clone.send(text.to_string()); // Fire-and-forget broadcast
                            }
                        }
                    }
                }
            }
        }
    });

    // Configure Axum web server with WebSocket support
    let app = Router::new()
        // Register WebSocket handler for /ws endpoint
        // When clients connect to ws://localhost:3001/ws, handle_ws function will be called
        .route("/ws", get(handle_ws))
        // Share application state across all request handlers
        // This state will be available in the handle_ws function
        .with_state((
            tx.clone(), // Clone of broadcast sender - allows sending messages to all connected WebSocket clients
            producer,   // Kafka producer instance - for publishing messages to Kafka topics
            seen_messages, // Thread-safe set of processed message IDs - prevents duplicate processing
        ));

    // Inform developer about server status
    println!("WebSocket server running at ws://localhost:3001/ws");

    // Start the web server
    axum::serve(
        // Create TCP listener that accepts connections on all network interfaces
        // "0.0.0.0:3001" means accessible from any IP address on port 3001
        tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap(),
        app, // Mount our application router
    )
    .await
    .unwrap(); // Server will run until manually stopped or an error occurs
}


/// WebSocket connection handler - upgrades HTTP requests to WebSocket connections
/// This function is called when a client initiates a WebSocket handshake at /ws
async fn handle_ws(
    // WebSocket upgrade request - provided by Axum automatically
    ws: WebSocketUpgrade,
    // Extract shared application state from the router
    State((tx, producer, seen_messages)): State<(
        broadcast::Sender<String>,  // For broadcasting messages to all clients
        FutureProducer,             // For sending messages to Kafka
        Arc<Mutex<HashSet<String>>>, // For tracking processed message IDs
    )>,
) -> impl IntoResponse {
    // Complete the WebSocket handshake and spawn the actual connection handler
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
