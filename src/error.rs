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

pub(crate) struct ActorReply<T> {
    result: ractor::rpc::CallResult<T>,
    label: &'static str,
}

impl<T> ActorReply<T> {
    pub(crate) fn new(result: ractor::rpc::CallResult<T>, label: &'static str) -> Self {
        Self { result, label }
    }

    pub(crate) fn into_result(self) -> Result<T> {
        match self.result {
            ractor::rpc::CallResult::Success(value) => Ok(value),
            ractor::rpc::CallResult::Timeout => {
                Err(Error::ActorCall(format!("{}: call timed out", self.label)))
            }
            ractor::rpc::CallResult::SenderError => Err(Error::ActorCall(format!(
                "{}: sender dropped before reply",
                self.label
            ))),
        }
    }
}
