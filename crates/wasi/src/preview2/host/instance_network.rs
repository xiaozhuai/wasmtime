use crate::preview2::bindings::sockets::instance_network;
use crate::preview2::network::Network;
use crate::preview2::WasiView;
use wasmtime::component::Resource;

impl instance_network::Host for WasiView {
    fn instance_network(&mut self) -> Result<Resource<Network>, anyhow::Error> {
        let network = Network {
            pool: self.ctx().pool.clone(),
            allow_ip_name_lookup: self.ctx().allow_ip_name_lookup,
        };
        let network = self.table.push(network)?;
        Ok(network)
    }
}
