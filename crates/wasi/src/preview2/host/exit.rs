use crate::preview2::{bindings::cli::exit, I32Exit, WasiView};

impl exit::Host for WasiView {
    fn exit(&mut self, status: Result<(), ()>) -> anyhow::Result<()> {
        let status = match status {
            Ok(()) => 0,
            Err(()) => 1,
        };
        Err(anyhow::anyhow!(I32Exit(status)))
    }
}
