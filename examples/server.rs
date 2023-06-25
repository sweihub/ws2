use log2::*;
use ws2::server::*;

fn main() -> anyhow::Result<()> {
    let _log2 = log2::start();
    let server = ws2::server::listen("127.0.0.1:3125")?;
    let mut n = 0;

    loop {
        match server.recv(0.5) {
            Event::Open(id, _sender, address) => {
                debug!("{id} on open: {address}");
            }
            Event::Close(id) => {
                debug!("{id} on close");
            }
            Event::Text(id, message) => {
                debug!("{id} on message: {message}");
                n += 1;
                if n >= 10 {
                    break;
                }
            }
            Event::Binary(id, _) => {
                debug!("{id} on binary");
            }
            Event::Timeout => {
                // debug!("timeout");
            }
            Event::Error(e) => {
                debug!("on error: {e}");
            }
        }
    }

    Ok(())
}
