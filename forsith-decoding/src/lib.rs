use std::io::Read;
use thiserror::Error;

/// Rough category of asset, like image/video.
pub enum AssetKind {
    Image,
    Audio,
    Video,
    Model,
}

pub trait Decoder: Sized + Iterator<Item = Result<Self::Chunk, DecodingError>> {
    type Chunk;

    const KIND: AssetKind;

    fn open<R: Read>(data: R) -> Result<Self, DecodingError>;
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Invalid header")]
    InvalidHeader()
}
