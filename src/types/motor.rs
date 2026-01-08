
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::collections::HashMap;

use crate::error::{ BuildError, CommunicationError };


/// Handles the behavioral output of a bionic neural network made with cajal.
/// When it receives a NeuronId, it executes the corresponding function.
/// `B` is the behavior function pointer, `A` is the argument for the function,
/// and `R` is the value returned by the function.
pub struct Motor<B: Fn(A) -> R, A, R> {

    /// The corresponding `cajal::io::Output` should be set to share this name.
    pub tract_name: String,

    /// This should be set up to match the address of the 
    /// corresponding `Output` to be read from.
    pub address: SocketAddr,
    pub(crate) socket: UdpSocket,

    /// Maps each NeuronId to a "behavior" function to execute 
    /// every time the NeuronId is received.
    /// These should correspond to those in `Output.senders`.
    /// The sender IDs can be retrieved with the `Output::sender_ids` method. 
    pub nerves: HashMap<u16, B>,
    phantom_data: std::marker::PhantomData<(A, R)>
} 

impl<B: Fn(A) -> R, A, R> Motor<B, A, R> {

    /// Create a motor socket. Use port '0' to have the system assign a port.
    /// The socket address will be recorded in the address field.
    pub async fn new(
        tract_name: &str,
        address: SocketAddr
    ) -> Result<Self, BuildError> {

        let mut motor = Motor {
            tract_name: tract_name.to_owned(),
            address,
            socket: UdpSocket::bind(address).await?,
            nerves: HashMap::new(),
            phantom_data: std::marker::PhantomData
        };

        motor.address = motor.socket.local_addr()?;
        Ok(motor)
    }

    /// Maps a neurotransmission signal to a process to be executed.
    /// NOTE: Overwrites existing NeuronId key without checking.
    pub fn add_nerve(&mut self, impulse: u16, behavior: B) {

        self.nerves.insert(impulse.clone(), behavior);
    }

    /// Receives NeuronId messages and executes the corresponding function.
    pub async fn recv_impulse(
        &self, 
        buffer: &mut [u8], 
        args: A
    ) -> Result<R, CommunicationError> {

        let n_bytes = self.socket.recv(buffer).await?;
        let buff = &buffer[..n_bytes];
        let impulse: u16 = bincode::deserialize_from(buff)?;

        if let Some(behavior) = self.nerves.get(&impulse) { 
            Ok(behavior(args)) 
        } else { 
            Err(CommunicationError::UnrecognizedImpulse(impulse))
        }
    }

}

