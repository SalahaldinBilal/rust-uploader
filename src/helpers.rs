use axum::body::BodyDataStream;
use futures::StreamExt;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::io;
use std::io::SeekFrom;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::io::AsyncRead;
use tokio::io::AsyncSeek;
use tokio::io::ReadBuf;
use tokio_util::bytes::BytesMut;

pub fn get_env_value(name: &str) -> String {
    std::env::var(name).expect(&format!("Env variable {} should exist", name))
}

pub fn get_file_extension(filename: &str) -> String {
    let split_name: Vec<&str> = filename.split(".").collect();

    if let Some(extension) = split_name.last() {
        return extension.to_string();
    } else {
        return "".to_string();
    }
}

pub fn create_jwt_token(secret: &str, claims: &BTreeMap<&str, &str>) -> Result<String, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret.as_bytes()).unwrap();
    claims.sign_with_key(&key)
}

pub fn verify_jwt_token(secret: &str, token: &str) -> Result<BTreeMap<String, String>, jwt::Error> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&secret.as_bytes()).unwrap();
    let claims: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);
    claims
}

pub struct AxumBodyStreamWrapper {
    stream: BodyDataStream,
    buffer: BytesMut,
    position: u64,
    stream_position: u64,
    seek_target: Option<u64>,
    seeking: bool,
}

unsafe impl Sync for AxumBodyStreamWrapper {}
unsafe impl Send for AxumBodyStreamWrapper {}

impl AxumBodyStreamWrapper {
    pub fn new(stream: BodyDataStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            position: 0,
            stream_position: 0,
            seek_target: None,
            seeking: false,
        }
    }
}

impl AsyncRead for AxumBodyStreamWrapper {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.position >= self.stream_position {
            match self.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    self.position = self.stream_position;
                    self.stream_position += chunk.len() as u64;
                    self.buffer.extend(chunk);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("body stream error: {:?}", e),
                    )));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        }

        if self.position >= self.stream_position {
            return Poll::Ready(Ok(()));
        }

        let start = (self.position - (self.stream_position - self.buffer.len() as u64)) as usize;
        let end = std::cmp::min(self.buffer.len(), start + buf.remaining());
        let to_read = end - start;

        if start < self.buffer.len() {
            buf.put_slice(&self.buffer[start..end]);
            self.position += to_read as u64;
        }

        Poll::Ready(Ok(()))
    }
}
impl AsyncSeek for AxumBodyStreamWrapper {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> io::Result<()> {
        match position {
            SeekFrom::Start(offset) => {
                self.seek_target = Some(offset);
            }
            SeekFrom::Current(offset) => {
                let new_pos = (self.position as i64).saturating_add(offset);
                if new_pos < 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Seeking before start of stream",
                    ));
                }
                self.seek_target = Some(new_pos as u64);
            }
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Seeking from end is not supported for streams",
                ));
            }
        }
        Ok(())
    }

    fn poll_complete(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
        if let Some(target) = self.seek_target.take() {
            self.position = target;
            Poll::Ready(Ok(self.position))
        } else {
            match (self.seeking, self.seek_target.is_some()) {
                (true, true) => Poll::Pending,
                (true, false) => {
                    self.seeking = false;
                    Poll::Ready(Ok(self.position))
                }
                _ => {
                    self.seeking = true;
                    Poll::Ready(Ok(self.position))
                }
            }
        }
    }
}
