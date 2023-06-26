use log2::*;
use ws2::client::*;

fn main() -> anyhow::Result<()> {
    let _log2 = log2::start();
    let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";

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
