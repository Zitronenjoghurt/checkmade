use crate::error::{CoreError, CoreResult};
use crate::game::session_data::normal::NormalSessionData;
use giga_chess::prelude::config::SessionConfig;

mod normal;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionData {
    Normal(NormalSessionData),
}

impl SessionData {
    #[cfg(all(feature = "brotli", feature = "rmp-serde", feature = "serde"))]
    pub fn from_bytes(compressed: &[u8]) -> CoreResult<Self> {
        use std::io::Read;
        let mut decompressed = Vec::new();
        brotli::Decompressor::new(compressed, 4096)
            .read_to_end(&mut decompressed)
            .map_err(|err| CoreError::SessionDeserialization(err.to_string()))?;

        rmp_serde::from_slice(&decompressed)
            .map_err(|err| CoreError::SessionDeserialization(err.to_string()))
    }

    #[cfg(all(feature = "brotli", feature = "rmp-serde", feature = "serde"))]
    pub fn to_bytes(&self) -> CoreResult<Vec<u8>> {
        use std::io::Write;
        let serialized = rmp_serde::to_vec_named(self)
            .map_err(|err| CoreError::SessionSerialization(err.to_string()))?;

        let mut compressed = Vec::new();
        let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 11, 22);
        writer
            .write_all(&serialized)
            .map_err(|err| CoreError::SessionSerialization(err.to_string()))?;
        drop(writer);

        Ok(compressed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionConfigData {
    Normal(SessionConfig),
}

impl SessionConfigData {
    #[cfg(all(feature = "rmp-serde", feature = "serde"))]
    pub fn from_bytes(raw: &[u8]) -> CoreResult<Self> {
        rmp_serde::from_slice(raw)
            .map_err(|err| CoreError::SessionConfigDeserialization(err.to_string()))
    }

    #[cfg(all(feature = "rmp-serde", feature = "serde"))]
    pub fn to_bytes(&self) -> CoreResult<Vec<u8>> {
        rmp_serde::to_vec_named(self)
            .map_err(|err| CoreError::SessionConfigSerialization(err.to_string()))
    }
}
