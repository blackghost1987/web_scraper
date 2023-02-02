use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("WebScraperError: {0}")]
    WebScraperError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    AddressError(#[from] std::net::AddrParseError),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    FloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    IntError(#[from] std::num::ParseIntError),
}

pub use Error::WebScraperError;

pub type Result<T> = std::result::Result<T, Error>;
