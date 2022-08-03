use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::io;


#[derive(Serialize, Deserialize, Clone)]
pub struct Asset {
    pub id: usize,
    pub name: String,
    pub category: String,
    pub vulnerabilities: usize,
    pub created_at: DateTime<Utc>,
}


#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

