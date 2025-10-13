//! Generated system module.
//!
//! This module contains the generated components and the system I/O.

use crossbeam_channel::{Sender, Receiver, bounded, SendError};

pub mod classification;
pub mod features_fusion;
pub mod lidar_detection;
pub mod object_tracking;
pub mod radar_detection;

pub struct Broadcast<T> {
    channels: Vec<Sender<T>>,
}

impl<T: 'static + Clone + Send + Sync> Broadcast<T> {
    pub fn new() -> Self {
        Self { channels: vec![] }
    }

    pub fn subscribe(&mut self) -> Receiver<T> {
        let (tx, rx) = bounded(1);

        self.channels.push(tx);

        rx
    }

    pub fn send(&self, message: T) -> Result<(), SendError<T>> {
        for c in self.channels.iter() {
            c.send(message.clone())?;
        }

        Ok(())
    }
}
