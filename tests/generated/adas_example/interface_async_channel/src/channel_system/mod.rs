//! Generated system module.
//!
//! This module contains the generated components and the system I/O.

use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};

pub mod classification;
pub mod features_fusion;
pub mod lidar_detection;
pub mod object_tracking;
pub mod radar_detection;

pub struct Broadcast<T> {
    channels: Vec<Sender<T>>,
}

impl<T: 'static + Clone + Send + Sync> Broadcast<T> {
    pub const fn new() -> Self {
        Self { channels: vec![] }
    }

    pub fn subscribe(&mut self) -> Receiver<T> {
        let (tx, rx) = channel(1);

        self.channels.push(tx);

        rx
    }

    pub async fn send(&self, message: T) -> Result<(), SendError<T>> {
        for c in self.channels.iter() {
            c.send(message.clone()).await?;
        }

        Ok(())
    }
}
