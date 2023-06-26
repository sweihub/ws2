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
