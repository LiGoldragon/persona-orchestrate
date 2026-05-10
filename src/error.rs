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

    #[error("system time: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("signal frame: {0}")]
    SignalFrame(#[from] signal_core::FrameError),

    #[error("signal persona mind: {0}")]
    SignalPersonaMind(#[from] signal_persona_mind::Error),

    #[error("nota: {0}")]
    Nota(#[from] nota_codec::Error),

    #[error("sema: {0}")]
    Sema(#[from] sema::Error),

    #[error("unexpected signal frame: {0}")]
    UnexpectedFrame(&'static str),

    #[error("missing signal auth proof")]
    MissingAuthProof,

    #[error("frame is larger than configured limit: {found} > {limit}")]
    FrameTooLarge { found: usize, limit: usize },

    #[error("missing command line input")]
    MissingCommandInput,

    #[error("unknown command line option: {option}")]
    UnknownCommandLineOption { option: String },

    #[error("missing value for command line option: {option}")]
    MissingCommandLineOptionValue { option: String },

    #[error("invalid command line argument: {argument}")]
    InvalidCommandLineArgument { argument: String },

    #[error("missing required --socket path")]
    MissingSocketPath,

    #[error("missing required --actor name")]
    MissingActorName,

    #[error("missing required --store path")]
    MissingStorePath,

    #[error("expected one NOTA request argument, got {count}")]
    WrongRequestArgumentCount { count: usize },
}
