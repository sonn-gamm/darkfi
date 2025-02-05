use async_std::sync::Arc;

use darkfi::{Error, Result};

use crate::model::Event;

pub type EventsQueuePtr = Arc<EventsQueue>;

pub struct EventsQueue(smol::channel::Sender<Event>, smol::channel::Receiver<Event>);

impl EventsQueue {
    pub fn new() -> EventsQueuePtr {
        let (sn, rv) = smol::channel::unbounded();
        Arc::new(Self(sn, rv))
    }

    pub async fn fetch(&self) -> Result<Event> {
        self.1.recv().await.map_err(Error::from)
    }

    pub async fn dispatch(&self, event: &Event) -> Result<()> {
        self.0.send(event.clone()).await.map_err(Error::from)
    }
}
