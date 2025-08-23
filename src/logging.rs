use std::io::{self, Write};
use std::thread;

use crossbeam_channel::{Sender, unbounded};
use tauri::Manager;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{Registry, reload};

struct GuiWriter {
    tx: Sender<String>,
}

impl Write for GuiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        let _ = self.tx.send(s);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn init_tracing_and_gui_emitter(app: tauri::AppHandle) -> reload::Handle<EnvFilter, Registry> {
    let (tx, rx) = unbounded::<String>();

    thread::spawn(move || {
        while let Ok(line) = rx.recv() {
            let _ = app.emit_all("log", line);
        }
    });

    let initial = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    let (filter_layer, handle) = reload::Layer::new(EnvFilter::new(initial));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().with_writer(move || GuiWriter { tx: tx.clone() }))
        .init();

    handle
}

pub fn raise_to_warn_if_release(
    handle: &reload::Handle<EnvFilter, Registry>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !cfg!(debug_assertions) {
        handle.reload(EnvFilter::new("warn"))?;
    }
    Ok(())
}
