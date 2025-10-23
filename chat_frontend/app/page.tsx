"use client"

import { useEffect, useState, useRef } from "react"
import { v4 as uuidv4 } from "uuid"
import { Users } from "lucide-react"

interface ChatMessage {
  sender_id: string
  username: string
  text: string
  timestamp: string
}

interface ActiveUser {
  sender_id: string
  username: string
  lastSeen: number
}

export default function ChatPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState("")
  const [username, setUsername] = useState("")
  const [senderId] = useState(() => uuidv4())
  const [activeUsers, setActiveUsers] = useState<Map<string, ActiveUser>>(new Map())
  const wsRef = useRef<WebSocket | null>(null)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  // Ask for username once
  useEffect(() => {
    const name = prompt("Enter your username:") || "Anonymous"
    setUsername(name)
  }, [])

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" })
  }, [messages])

  // Connect WebSocket
  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3001/ws")
    wsRef.current = ws

    ws.onopen = () => console.log("Connected to WebSocket")
    ws.onclose = () => console.log("WebSocket disconnected")

    ws.onmessage = (event) => {
      try {
        const msg: ChatMessage = JSON.parse(event.data)
        if (msg.text && msg.sender_id) {
          setMessages((prev) => [...prev, msg])

          setActiveUsers((prev) => {
            const updated = new Map(prev)
            updated.set(msg.sender_id, {
              sender_id: msg.sender_id,
              username: msg.username,
              lastSeen: Date.now(),
            })
            return updated
          })
        }
      } catch {
        console.warn("Non-JSON message ignored:", event.data)
      }
    }

    return () => ws.close()
  }, [])

  useEffect(() => {
    const interval = setInterval(() => {
      setActiveUsers((prev) => {
        const updated = new Map(prev)
        const now = Date.now()
        for (const [key, user] of updated.entries()) {
          if (now - user.lastSeen > 5 * 60 * 1000) {
            updated.delete(key)
          }
        }
        return updated
      })
    }, 30000)

    return () => clearInterval(interval)
  }, [])

  const sendMessage = () => {
    const text = input.trim()
    if (!text || !wsRef.current) return

    const msg = {
      sender_id: senderId,
      username,
      text,
    }

    wsRef.current.send(JSON.stringify(msg))
    setInput("")
  }

  const formatTime = (iso: string) => {
    const date = new Date(iso)
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
  }

  const sortedUsers = Array.from(activeUsers.values()).sort((a, b) => a.username.localeCompare(b.username))

  return (
    <div className="h-screen flex flex-col bg-background">
      <div className="border-b bg-card px-6 py-4">
        <h1 className="text-2xl font-bold text-foreground">Kafka Chat</h1>
        <p className="text-sm text-muted-foreground">Rust + Next.js</p>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Chat area */}
        <div className="flex-1 flex flex-col">
          {/* Messages container */}
          <div className="flex-1 overflow-y-auto p-6 space-y-4">
            {messages.length === 0 ? (
              <div className="flex items-center justify-center h-full">
                <p className="text-muted-foreground text-lg">No messages yet. Start the conversation!</p>
              </div>
            ) : (
              <>
                {messages.map((msg, idx) => {
                  const isMine = msg.sender_id === senderId
                  return (
                    <div key={idx} className={`flex ${isMine ? "justify-end" : "justify-start"}`}>
                      <div
                        className={`px-4 py-3 rounded-2xl max-w-[60%] ${
                          isMine
                            ? "bg-primary text-primary-foreground rounded-br-none"
                            : "bg-secondary text-secondary-foreground rounded-bl-none"
                        }`}
                      >
                        <div className="text-sm font-semibold">{isMine ? "You" : msg.username}</div>
                        <div className="mt-1">{msg.text}</div>
                        {msg.timestamp && (
                          <div
                            className={`text-xs ${
                              isMine ? "text-primary-foreground/70" : "text-secondary-foreground/70"
                            } text-right`}
                          >
                            {formatTime(msg.timestamp)}
                          </div>
                        )}
                      </div>
                    </div>
                  )
                })}
                <div ref={messagesEndRef} />
              </>
            )}
          </div>

          {/* Input area */}
          <div className="border-t bg-card p-6">
            <div className="flex gap-3">
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Type a message..."
                className="flex-1 border border-input rounded-2xl px-4 py-3 focus:outline-none focus:ring-1 focus:ring-primary bg-background text-foreground"
                onKeyDown={(e) => e.key === "Enter" && sendMessage()}
              />
              <button
                onClick={sendMessage}
                className="bg-primary text-primary-foreground px-6 py-3 rounded-2xl hover:opacity-90 transition font-medium"
              >
                Send
              </button>
            </div>
          </div>
        </div>

        <div className="w-64 border-l bg-card border-border flex flex-col">
          <div className="p-4 border-b border-border">
            <div className="flex items-center gap-2">
              <Users className="w-5 h-5 text-primary" />
              <h2 className="font-semibold text-foreground">Active Users ({sortedUsers.length})</h2>
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-4 space-y-2">
            {sortedUsers.length === 0 ? (
              <p className="text-sm text-muted-foreground text-center py-8">No active users</p>
            ) : (
              sortedUsers.map((user) => (
                <div
                  key={user.sender_id}
                  className="flex items-center gap-3 p-3 rounded-lg bg-background hover:bg-muted transition"
                >
                  <div className="w-2 h-2 rounded-full bg-green-500" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground truncate">
                      {user.sender_id === senderId ? "You" : user.username}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
