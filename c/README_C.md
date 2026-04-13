# ZeroMQ C Testing App

A lightweight ZeroMQ test application written in C.

## Requirements

- **GCC**: C compiler
- **libzmq-dev**: ZeroMQ development headers and libraries

To install dependencies on Debian/Ubuntu:
```bash
sudo apt update && sudo apt install build-essential libzmq3-dev
```

## Compilation

Build the application using the provided Makefile:

```bash
make
```

This will produce an executable named `zmq_test`.

## Usage

The application supports `server` and `client` modes.

### Running the Server

Acts as a REP (Reply) socket that listens for messages.

```bash
./zmq_test server [address]
```

**Example:**
```bash
./zmq_test server tcp://*:5555
```

### Running the Client

Acts as a REQ (Request) socket that sends messages to the server.

```bash
./zmq_test client [address] [name] [count]
```

**Example:**
```bash
./zmq_test client tcp://localhost:5555 James 10
```

## Cleanup

To remove the compiled binary:
```bash
make clean
```
