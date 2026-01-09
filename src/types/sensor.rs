
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::hash::Hash;

use crate::error::{ BuildError, CommunicationError };


/// Sends some data impulse as a NeuronId to trigger a Complex's Inputs.
/// The frequency of that data's occurrence should form a meaningful signal.
/// `Q` is the quantized form of the datum that creates the signal impulse.
/// `Q` can also be any post-conversion key for triggering the impulse.
pub struct Sensor<Q: Hash + Eq> {

    /// The corresponding `cajal::io::Input` should be set to share this name. 
    pub tract_name: String,

    /// This should be set up to match the address of the 
    /// corresponding `Input` that will read the Sensor signal.
    pub address: SocketAddr,
    pub(crate) socket: UdpSocket,

    /// These should correspond to the NeuronIds in `Input.fibers`.
    /// The fiber IDs can be retrieved with the `Input::fiber_ids` method. 
    pub spectrum: HashMap<Q, u16>,
} 

impl<Q: Hash + Eq> Sensor<Q> {

    /// Create a sensor socket. Use port '0' to have the system assign a port.
    /// The socket address will be recorded in the address field.
    pub async fn new(
        tract_name: &str,
        address: SocketAddr
    ) -> Result<Self, BuildError> {

        let mut sensor = Sensor {
            tract_name: tract_name.to_owned(),
            address,
            socket: UdpSocket::bind(address).await?,
            spectrum: HashMap::new()
        };

        sensor.address = sensor.socket.local_addr()?;
        Ok(sensor)
    }

    /// Maps a sensory bit to a new NeuronId.
    /// NOTE: Overwrites existing quantum key without checking.
    pub fn add_receptor(&mut self, quantum: Q, fid: u16) {

        self.spectrum.insert(quantum, fid);
    }

    /// Connect to a remote socket. 
    /// Remember to ensure that the corresponding Input
    /// can handle all fiber IDs that will be sent by this sensor.
    pub async fn connect(
        &mut self, 
        remote: &SocketAddr
    ) -> Result<(), BuildError> {

        self.socket.connect(remote).await?;
        self.address = remote.to_owned();
        Ok(())
    }

    /// Attempts to send a sensory datum as a neurotransmission impulse.
    pub async fn send_impulse(
        &self, 
        quantum: &Q
    ) -> Result<(), CommunicationError> {

        if let Some(nid) = self.spectrum.get(quantum) {
            let nid = bincode::serialize(nid)?;
            self.socket.send(&nid).await?;
            Ok(())
        } else { 
            let name = self.tract_name.clone();
            Err(CommunicationError::UnrecognizedTrigger(name)) 
        }
    }

}


use cajal_cx::tract::{ Tract, sender::TractSender };

impl<Q: Hash + Eq> Tract for Sensor<Q> {
    fn tract_name(&self) -> &str { &self.tract_name }
    fn num_fibers(&self) -> usize { self.spectrum.len() }
    fn tract_address(&self) -> SocketAddr { self.address.clone() }
}

impl<Q: Hash + Eq> TractSender for Sensor<Q> {

    async fn set_target_address(&mut self, target_address: SocketAddr) -> Result<(), std::io::Error> {
        self.socket.connect(target_address).await?;
        self.address = target_address.clone();
        Ok(())
    }
}
