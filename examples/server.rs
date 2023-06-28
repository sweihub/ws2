use log2::*;
use ws2::{Pod, WebSocket};

struct Worker;

impl ws2::Handler for Worker {
    fn on_open(&mut self, ws: &WebSocket) -> Pod {
        info!("on open: {ws}");
        Ok(())
    }

    fn on_close(&mut self, ws: &WebSocket) -> Pod {
        info!("on close: {ws}");
        Ok(())
    }

    fn on_message(&mut self, ws: &WebSocket, msg: String) -> Pod {
        info!("on message: {msg}, {ws}");
        let echo = format!("echo: {msg}");
        let n = ws.send(echo);
        Ok(n?)
    }
}

fn main() -> Pod {
    let _log2 = log2::start();
    let address = "127.0.0.1:3125";
    let mut worker = Worker {};

    info!("listen on: {address}");
    let mut server = ws2::listen(address)?;

    loop {
        let _ = server.process(&mut worker, 0.5);
        // do other stuff
    }
}
