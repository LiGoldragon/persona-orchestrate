use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("actor call: {0}")]
    ActorCall(String),

    #[error("actor spawn: {0}")]
    ActorSpawn(String),

    #[error("actor join: {0}")]
    ActorJoin(String),
}
