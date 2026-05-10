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

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("signal frame: {0}")]
    SignalFrame(#[from] signal_core::FrameError),

    #[error("unexpected signal frame: {0}")]
    UnexpectedFrame(&'static str),

    #[error("frame is larger than configured limit: {found} > {limit}")]
    FrameTooLarge { found: usize, limit: usize },
}
