# Video Feedback Widget

A  rust project I built 5 years ago that demonstrates modern web application development using WebAssembly and actix. This project was built as an exploration into Rust's ecosystem, particularly focusing on web development, real-time communication, and video processing capabilities. I had little to no knowledge of rust at the time but thought i would share.

The application enables users to record, pause, and submit video feedback with seamless device management and clip handling. 

It serves as a example of building complex, interactive web applications in Rust while learning the language's features. While some of the dependencies and approaches may be outdated since this was something i built 5 years ago, the core concepts and learning journey remain valuable.

https://github.com/user-attachments/assets/54eac4f0-c070-4984-a6b9-65b9415aea9d

## Features

- **Device Management**
  - Custom microphone and camera selection
  - Persistent device preferences
  - Permission handling

- **Recording Controls**
  - Pause/Resume functionality
  - Multi-clip recording support
  - Real-time preview
  - Clip discard support

- **Backend Processing**
  - WebSocket-based clip upload
  - Automatic clip merging
  - Video encoding optimization

## Architecture

### Backend Components
- `controllers/`: Handles HTTP and WebSocket request routing
  - `clips.rs`: Manages video clip processing and storage
- `helpers/`: Utility functions and error handling
  - `errors.rs`: Custom error types and handling
  - `utilities.rs`: Shared utility functions
- `services/`: Core business logic
  - `ffmpeg.rs`: Video processing and encoding

### Frontend Components
- `models/`: Data structures and state management
- `service/`: External service integrations
  - `camera.rs`: Device handling and video capture
  - `feedback.rs`: Feedback submission logic
  - `web_socket.rs`: WebSocket communication
- Core components:
  - `camera.rs`: Camera interface component
  - `feedback.rs`: Main feedback widget
  - `form.rs`: Form handling
  - `loading_animated.rs`: Loading animations
  - `utilities.rs`: Shared utilities

## Tech Stack

### Frontend
- Rust
- Yew (WebAssembly framework)
- WebRTC APIs
- MediaRecorder API

### Backend
- Rust
- Actix web framework
- WebSocket protocol
- Video processing libraries

## Prerequisites

### System Requirements 🛠️

You'll need FFmpeg installed on your system - it's what handles all the video processing magic:

- FFmpeg version 4.0 or higher
- libx264 for video encoding
- libmp3lame for audio encoding

Quick install:
```bash
# On Ubuntu/Debian
sudo apt-get update
sudo apt-get install ffmpeg

# On macOS with Homebrew
brew install ffmpeg

# On Windows with Chocolatey
choco install ffmpeg
```

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install WebAssembly target
rustup target add wasm32-unknown-unknown

# Install trunk (Yew development server)
cargo install trunk

# Install cargo-watch (optional, for development)
cargo install cargo-watch
```

## Project Structure

```
├── backend/
│   ├── src/
│   │   ├── controllers/
│   │   │   ├── clips.rs
│   │   │   └── mod.rs
│   │   ├── helpers/
│   │   │   ├── errors.rs
│   │   │   ├── mod.rs
│   │   │   └── utilities.rs
│   │   ├── services/
│   │   │   ├── ffmpeg.rs
│   │   │   └── mod.rs
│   │   └── main.rs
│   ├── .env
│   ├── .gitignore
│   ├── Cargo.lock
│   └── Cargo.toml
└── frontend/
    ├── src/
    │   ├── models/
    │   │   └── mod.rs
    │   ├── service/
    │   │   ├── camera.rs
    │   │   ├── feedback.rs
    │   │   ├── mod.rs
    │   │   └── web_socket.rs
    │   ├── camera.rs
    │   ├── feedback.rs
    │   ├── form.rs
    │   ├── lib.rs
    │   ├── loading_animated.rs
    │   └── utilities.rs
    ├── .gitignore
    ├── Cargo.lock
    ├── Cargo.toml
    ├── index.html
    └── main.css
```

## Getting Started

1. Start the backend server:
```bash
cd backend
cargo run
```

2. In a new terminal, start the frontend development server:
```bash
cd frontend
trunk serve --port=42069
```

3. Open your browser and navigate to `http://localhost:42069`

## Development

### Backend Development

```bash
# Run with auto-reload
cargo watch -x run

# Build for release
cargo build --release
```

### Frontend Development

```bash
# Run development server
trunk serve --port=42069

# Build for production
trunk build --release
```

## Configuration

Environment variables:

```env
TEMP_DIRECTORY="/home/[user]/temp"
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

