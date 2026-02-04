use std::env;

use crate::MSIC;
use tokio::net::UnixStream;

pub mod model;
pub use model::*;
pub mod reader;
pub use reader::{IReader, IReaderError};
pub mod writer;
pub use writer::{IWriter, IWriterError};

pub const DATA_PATH_ENV_VAR: &str = "DATA_PATH";

pub async fn connect() -> Result<(IWriter, IReader), IWriterError> {
    let stream = UnixStream::connect("igloo.sock").await?;

    let (reader, writer) = stream.into_split();
    let mut writer = IWriter::new(writer);
    let reader = IReader::new(reader);

    writer.whats_up_igloo(MSIC, MSIM).await?;
    writer.flush().await?;

    Ok((writer, reader))
}

pub fn get_data_path() -> String {
    env::var(DATA_PATH_ENV_VAR).unwrap()
}
