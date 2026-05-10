use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::StoreLocation;

pub(super) struct Config {
    store: StoreLocation,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: StoreLocation,
}

#[allow(dead_code)]
struct ReadStoreLocation {
    probe: StoreLocationProbe,
}

#[allow(dead_code)]
impl ReadStoreLocation {
    fn expecting(store: StoreLocation) -> Self {
        Self {
            probe: StoreLocationProbe {
                expected: Some(store),
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct StoreLocationProbe {
    expected: Option<StoreLocation>,
}

impl Config {
    fn new(store: StoreLocation) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &StoreLocation {
        &self.store
    }
}

impl Actor for Config {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments.store))
    }
}

impl Message<ReadStoreLocation> for Config {
    type Reply = StoreLocation;

    async fn handle(
        &mut self,
        message: ReadStoreLocation,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let _matches = match message.probe.expected.as_ref() {
            Some(expected) => expected == self.store(),
            None => true,
        };
        self.store().clone()
    }
}
