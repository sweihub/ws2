# Very Simple Websocket For Rust

`ws2` is a very easy to use WebSocket server & client for Rust, builds on `ws` crate. it was designed for production, the receive timeout provides a non-blocking way.

## Server Example

```rust
use log2::*;
use ws2::{Pod, WebSocket};

struct Worker;

impl ws2::Handler for Worker {
    fn on_open(&self, ws: &WebSocket) -> Pod {
        info!("on open: {ws}");
        Ok(())
    }

    fn on_close(&self, ws: &WebSocket) -> Pod {
        info!("on close: {ws}");
        Ok(())
    }

    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
        info!("on message: {msg}, {ws}");
        let echo = format!("echo: {msg}");
        let n = ws.send(echo);
        Ok(n?)
    }
}

fn main() -> Pod {
    let _log2 = log2::start();
    let mut server = ws2::server::listen("127.0.0.1:3125")?;
    let worker = Worker {};

    loop {
        let _ = server.process(&worker, 0.5);
        // do other stuff
    }
}
```

## Client Example

The websocket client was designed as out-of-the-box, it will auto reconnect every 3s.

```rust
use log2::*;
use ws2::{Pod, WebSocket};

struct Worker;

impl ws2::Handler for Worker {
    fn on_open(&self, ws: &WebSocket) -> Pod {
        // ws.send("Hello World")?;
        info!("on open: {ws}");
        Ok(())
    }

    fn on_close(&self, ws: &WebSocket) -> Pod {
        info!("on close: {ws}");
        Ok(())
    }

    fn on_message(&self, ws: &WebSocket, msg: String) -> Pod {
        info!("on message: {msg}, {ws}");
        Ok(())
    }
}

fn main() -> Pod {
    let _log2 = log2::start();
    let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";
    let mut client = ws2::client::connect(url)?;
    let workder = Worker {};

    loop {
        let _ = client.process(&workder, 0.5);
        // do other stuff
    }
}
```
