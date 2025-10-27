# WebSocket Chat Application
This is a real-time chat application built with a Rust backend (Axum + Kafka) and a Next.js/React frontend. 
The features include real-time messaging, active user tracking, and persistent message storage.

## Features
**Real-time Messaging:** Instant message delivery via WebSockets

**Active User Tracking:** See who's currently online with live status updates

**Message Persistence:** All messages stored in Kafka for durability

**Modern UI:** Clean, responsive interface with message timestamps

**User Identification:** Unique usernames with session management

**Auto-scroll:** Automatic scrolling to the latest messages

## Architecture
```
Frontend (Next.js/React)
        ↓
WebSocket Connection
        ↓
Rust Backend (Axum)
        ↓
Apache Kafka
```

## Prerequisites
  -Node.js 18+ and npm
  
  -Rust and Cargo

  -Apache Kafka running on localhost:9092

## Installation & Setup
### 1. Backend Setup
Clone the repository
```
git clone git@github.com:cosmas-ships/kafka_rust_chat.git
cd kafka_rust_chat
```

Start the Rust backend
```
cd backend
cargo run
```
Backend will start on: ws://localhost:3001

### 2. Frontend Setup
In a new terminal, navigate to frontend
```
cd frontend
```
Install dependencies
```
npm install
```
Start the development server
```
npm run dev
```
Frontend will start on: 
```
http://localhost:3000
```
### 3. Kafka Setup
Make sure Kafka is running and create the required topic:

```
kafka-topics.sh --create --topic chat-room --bootstrap-server localhost:9092
```
## Usage
Access the Application: Open 
```
http://localhost:3000
```
in your browser

Set Username: Enter your username when prompted

Start Chatting: Type messages and press Enter or click Send

View Active Users: See online users in the right sidebar

## Features in Detail
-Real-time Messaging
Messages appear instantly for all connected users

-Timestamps show when each message was sent

-Visual distinction between your messages and others'

-Active User Tracking
Green status indicators show online users

-Users automatically removed after 5 minutes of inactivity

-Real-time updates when users join or leave

Message Format
```
interface ChatMessage {
  sender_id: string    // Unique user identifier
  username: string     // Display name
  text: string        // Message content
  timestamp: string   // ISO timestamp
}
```
## Configuration
Backend Configuration
WebSocket Port: 3001

Kafka Server: localhost:9092

Kafka Topic: chat-room

Frontend Configuration
WebSocket URL: ws://localhost:3001/ws

Development Server: http://localhost:3000

## Project Structure
```
websocket-chat-app/
├── backend/
│   ├── src/
│   │   └── main.rs          # Rust WebSocket server
│   └── Cargo.toml
└── frontend/
    ├── src/
    │   └── app/
    │       └── page.tsx     # React chat component
    ├── package.json
    └── next.config.js
```
## UI Components
**Message Bubbles**: Distinct styling for sent vs received messages

**User Sidebar:** Real-time active user list with status indicators

**Input Area:** Clean message input with send button

**Header:** Application title and description

## Message Flow
User types message → Frontend sends via WebSocket

Backend receives message → Adds UUID and timestamp

Message published to Kafka → Stored for persistence

Backend broadcasts to all clients → Real-time delivery

Frontend displays message → Updates UI immediately

## Development
Backend Dependencies
```
axum: Web framework with WebSocket support

rdkafka: Kafka client for Rust

tokio: Async runtime

uuid: Unique ID generation

chrono: Timestamp handling
```
Frontend Dependencies
```
React: UI framework

WebSocket API: Native browser WebSocket

uuid: Unique ID generation

Lucide React: Icons
```

## Troubleshooting
### Common Issues

WebSocket Connection Failed

- Ensure backend is running on port 3001

- Check ws://localhost:3001/ws is accessible

Kafka Connection Issues

- Verify Kafka is running on localhost:9092

- Confirm chat-room topic exists

Frontend Build Errors

- Clear node_modules and reinstall dependencies

- Check Node.js version compatibility

## Future Enhancements
- Message history loading

- Private messaging

- File sharing

- Message reactions

- User avatars

- Chat rooms/channels

- Message search

- Mobile app

## Contributing
-Fork the repository

-Create a feature branch

-Commit your changes

-Push to the branch

-Open a Pull Request

## License
This project is licensed under the MIT License - see the LICENSE file for details.

#### Happy Chatting! 🎉
