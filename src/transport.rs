use std::fs;
use std::path::{Path, PathBuf};

use signal_core::{Reply, Request};
use signal_persona_mind::{ActorName, Frame, FrameBody, MindReply, MindRequest};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

use crate::{
    Error, MindEnvelope, MindRoot, MindRootArguments, Result, StoreLocation, SubmitEnvelope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindDaemonEndpoint {
    socket: PathBuf,
}

impl MindDaemonEndpoint {
    pub fn new(socket: impl Into<PathBuf>) -> Self {
        Self {
            socket: socket.into(),
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.socket
    }

    fn bind_listener(&self) -> Result<UnixListener> {
        match fs::remove_file(&self.socket) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(UnixListener::bind(&self.socket)?)
    }

    fn remove_socket(&self) -> Result<()> {
        match fs::remove_file(&self.socket) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MindFrameCodec {
    maximum_frame_bytes: usize,
}

impl MindFrameCodec {
    pub const fn new(maximum_frame_bytes: usize) -> Self {
        Self {
            maximum_frame_bytes,
        }
    }

    pub async fn read_frame(&self, stream: &mut UnixStream) -> Result<Frame> {
        let mut prefix = [0_u8; 4];
        stream.read_exact(&mut prefix).await?;
        let length = u32::from_be_bytes(prefix) as usize;
        if length > self.maximum_frame_bytes {
            return Err(Error::FrameTooLarge {
                found: length,
                limit: self.maximum_frame_bytes,
            });
        }

        let mut bytes = Vec::with_capacity(4 + length);
        bytes.extend_from_slice(&prefix);
        bytes.resize(4 + length, 0);
        stream.read_exact(&mut bytes[4..]).await?;
        Ok(Frame::decode_length_prefixed(&bytes)?)
    }

    pub async fn write_frame(&self, stream: &mut UnixStream, frame: &Frame) -> Result<()> {
        let bytes = frame.encode_length_prefixed()?;
        stream.write_all(&bytes).await?;
        stream.flush().await?;
        Ok(())
    }

    pub fn request_frame(&self, actor: &ActorName, request: MindRequest) -> Frame {
        let _ingress_scaffold = actor;
        Frame::new(FrameBody::Request(Request::assert(request)))
    }

    pub fn reply_frame(&self, reply: MindReply) -> Frame {
        Frame::new(FrameBody::Reply(Reply::operation(reply)))
    }

    pub fn request_from_frame(&self, frame: Frame) -> Result<MindRequest> {
        match frame.into_body() {
            FrameBody::Request(Request::Operation { payload, .. }) => Ok(payload),
            _ => Err(Error::UnexpectedFrame("expected mind request operation")),
        }
    }

    pub fn reply_from_frame(&self, frame: Frame) -> Result<MindReply> {
        match frame.into_body() {
            FrameBody::Reply(Reply::Operation(reply)) => Ok(reply),
            _ => Err(Error::UnexpectedFrame("expected mind reply operation")),
        }
    }
}

impl Default for MindFrameCodec {
    fn default() -> Self {
        Self::new(1024 * 1024)
    }
}

pub struct MindClient {
    endpoint: MindDaemonEndpoint,
    actor: ActorName,
    codec: MindFrameCodec,
}

impl MindClient {
    pub fn new(endpoint: MindDaemonEndpoint, actor: ActorName) -> Self {
        Self {
            endpoint,
            actor,
            codec: MindFrameCodec::default(),
        }
    }

    pub async fn submit(&self, request: MindRequest) -> Result<MindReply> {
        let mut stream = UnixStream::connect(self.endpoint.as_path()).await?;
        let frame = self.codec.request_frame(&self.actor, request);
        self.codec.write_frame(&mut stream, &frame).await?;
        let reply = self.codec.read_frame(&mut stream).await?;
        self.codec.reply_from_frame(reply)
    }
}

pub struct MindDaemon {
    endpoint: MindDaemonEndpoint,
    store: StoreLocation,
    codec: MindFrameCodec,
}

impl MindDaemon {
    pub fn new(endpoint: MindDaemonEndpoint, store: StoreLocation) -> Self {
        Self {
            endpoint,
            store,
            codec: MindFrameCodec::default(),
        }
    }

    pub async fn bind(self) -> Result<BoundMindDaemon> {
        let listener = self.endpoint.bind_listener()?;
        let root = MindRoot::start(MindRootArguments::new(self.store)).await?;
        Ok(BoundMindDaemon {
            endpoint: self.endpoint,
            codec: self.codec,
            listener,
            root,
        })
    }
}

pub struct BoundMindDaemon {
    endpoint: MindDaemonEndpoint,
    codec: MindFrameCodec,
    listener: UnixListener,
    root: crate::ActorRef<MindRoot>,
}

impl BoundMindDaemon {
    pub fn endpoint(&self) -> &MindDaemonEndpoint {
        &self.endpoint
    }

    pub async fn serve_one(self) -> Result<MindReply> {
        let reply = self.serve_next().await;
        MindRoot::stop(self.root).await?;
        self.endpoint.remove_socket()?;
        reply
    }

    pub async fn serve_count(self, count: usize) -> Result<Vec<MindReply>> {
        let mut replies = Vec::with_capacity(count);
        let result = async {
            for _ in 0..count {
                replies.push(self.serve_next().await?);
            }
            Ok(replies)
        }
        .await;
        MindRoot::stop(self.root).await?;
        self.endpoint.remove_socket()?;
        result
    }

    pub async fn serve_forever(self) -> Result<()> {
        loop {
            if let Err(error) = self.serve_next().await {
                eprintln!("mind daemon client error: {error}");
            }
        }
    }

    async fn serve_next(&self) -> Result<MindReply> {
        let (mut stream, _address) = self.listener.accept().await?;
        let frame = self.codec.read_frame(&mut stream).await?;
        let actor = ActorName::new("operator");
        let request = self.codec.request_from_frame(frame)?;
        let envelope = MindEnvelope::new(actor, request);
        let root_reply = self
            .root
            .ask(SubmitEnvelope { envelope })
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        let reply = root_reply
            .reply()
            .cloned()
            .ok_or(Error::UnexpectedFrame("mind root returned no reply"))?;
        let frame = self.codec.reply_frame(reply.clone());
        self.codec.write_frame(&mut stream, &frame).await?;
        Ok(reply)
    }
}
