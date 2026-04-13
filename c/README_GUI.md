# ZeroMQ GUI Testing App (PUB/SUB over IPC)

A graphical ZeroMQ test application using the **Publish-Subscribe** pattern with IPC sockets.

## Pattern: PUB/SUB

In this version:
- **Server (Publisher)**: Can send messages to all connected clients.
- **Client (Subscriber)**: Receives and displays all messages published by the server.

## Requirements

- **GTK+ 3 Development Headers**
- **pkg-config**

To install on Debian/Ubuntu:
```bash
sudo apt update && sudo apt install libgtk-3-dev pkg-config
```

## Compilation

Build the application:
```bash
make
```

## Usage

### 1. Running the Publisher (Server)
The publisher binds to the IPC socket and allows you to send messages.
```bash
./zmq_gui server
```
Use the text entry and **"Publish"** button to send messages.

### 2. Running the Subscriber (Client)
The subscriber connects to the IPC socket and displays any incoming messages.
```bash
./zmq_gui client
```
You can open **multiple subscriber windows**, and they will all receive the same messages from the publisher.

## Configuration
Default IPC Path: `ipc:///tmp/zmqtest.ipc`
