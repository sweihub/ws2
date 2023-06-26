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
    let mut client = ws2::connect(url)?;
    let workder = Worker {};

    loop {
        let _ = client.process(&workder, 0.5);
        // do other stuff
    }
}
