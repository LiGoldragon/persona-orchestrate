use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

use crate::StoreLocation;

pub(super) struct Config;

pub struct State {
    store: StoreLocation,
}

pub struct Arguments {
    pub store: StoreLocation,
}

#[allow(dead_code)]
pub enum Message {
    StoreLocation {
        reply_port: RpcReplyPort<StoreLocation>,
    },
}

impl State {
    pub fn new(store: StoreLocation) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &StoreLocation {
        &self.store
    }
}

#[ractor::async_trait]
impl Actor for Config {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(State::new(arguments.store))
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::StoreLocation { reply_port } => {
                let _ = reply_port.send(state.store().clone());
            }
        }
        Ok(())
    }
}
