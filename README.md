# RESP Parser & Valkey Server

## Overview

This project is a Rust-based implementation of a RESP (Redis Serialization Protocol) parser along with a simple server module named Valkey. The purpose of this implementation is to handle RESP data structures, which are commonly used in Redis communication.

## Features

- Implements a RESP parser capable of handling different RESP data types.

- Provides a Valkey module that can act as a lightweight Redis-like server.

- Uses Rust's standard I/O operations for parsing and network communication.

## Project Structure

- parser.rs: Contains the RespParser and RespValue implementations for handling RESP messages.

- valkey.rs: Implements the Valkey server, which listens on a given address and processes incoming RESP requests.

- main.rs: Entry point that initializes and runs the Valkey server.

## Installation & Setup

### Prerequisites

- Rust (latest stable version recommended)

- Cargo (Rust's package manager)

### Steps to Build and Run

1. Clone the repository:

```
git clone <repository-url>
cd <project-directory>
```

2. Build the project:
```
cargo build --release
```

3. Run the server:
```
cargo run
```
By default, the server listens on 127.0.0.1:6379.

## Usage

The server will start listening on the specified address and handle RESP messages.

You can connect using a Redis client or a simple TCP client to send RESP-formatted requests.

