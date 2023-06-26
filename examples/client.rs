use log2::*;
use ws2::{client::*, Pod, WebSocket};

#[allow(unused)]
fn main1() -> Pod {
    let _log2 = log2::start();
    let url = "ws://127.0.0.1:3125";

    let mut ws = ws2::client::connect(url)?;
    let mut n = 0;

    loop {
        match ws.recv(0.5) {
            Event::Open(_) => {
                info!("on open");
            }
            Event::Close => {
                info!("on close");
            }
            Event::Text(msg) => {
                info!("on message: {msg}");
            }
            Event::Binary(_) => {}
            Event::Timeout => {
                info!("timeout");
            }
            Event::Error(e) => {
                error!("error: {e}");
                n += 1;
                if n >= 10 {
                    break;
                }
            }
        }
    }

    Ok(())
}

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
    let mut ws = ws2::client::connect(url)?;
    let workder = Worker {};

    loop {
        let _ = ws.process(&workder, 0.5);
        // do other stuff
    }
}
