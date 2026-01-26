use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

// 默认页码
const DEFAULT_PAGE: u64 = 1;

// 默认数据条数
const DEFAULT_SIZE: u64 = 5;


#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrNumber<T> {
    String(String),
    Number(T),
}

fn deserialize_number<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
T: FromStr + Deserialize<'de>,
T::Err: Display,
D: serde::Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse().map_err(serde::de::Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

fn default_page() -> u64 {
    DEFAULT_PAGE
}

fn default_size() -> u64 {
    DEFAULT_SIZE
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PaginationParams{
    #[serde(default="default_page", deserialize_with = "deserialize_number")]
    pub page: u64,

    #[serde(default="default_size", deserialize_with = "deserialize_number")]
    pub size: u64
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub page: u64,
    pub size: u64,
    pub total: u64,
    pub items: Vec<T>,
}

impl<T> Page<T> {
    pub fn new(page: u64, size: u64, total: u64, items: Vec<T>) -> Self {
        Self {page, size, total, items}
    }

    pub fn from_pagination(pagination: PaginationParams, total: u64, items: Vec<T>) -> Self {
        Self::new(pagination.page, pagination.size, total, items)
    }
}