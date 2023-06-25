use log2::*;
use ws2::client::*;

fn main() -> anyhow::Result<()> {
    let _log2 = log2::start();

    //let url = "wss://stream.binance.com:9443/ws/btcusdt@miniTicker";
    let url = "ws://172.16.210.210:5600";

    let mut ws = ws2::client::connect(url)?;
    let mut n = 0;

    loop {
        match ws.recv(0.5) {
            Event::Open(_) => {
                info!("on open");
                let _ = ws.send("Hello World");
            }
            Event::Close => {
                info!("on close");
            }
            Event::Text(msg) => {
                n += 1;
                info!("on message: {msg}");
                if n >= 10 {
                    ws.close();
                }
            }
            Event::Binary(_) => {}
            Event::Timeout => {
                info!("timeout");
            }
            Event::Error(e) => {
                error!("error: {e}");
                //break;
            }
        }
    }

    Ok(())
}
