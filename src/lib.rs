use crate::error::*;
use log::*;
use std::collections::HashMap;

pub mod error;
pub mod scrapers;

pub type QueryParam = (String, String);
pub type QuerySplitter<'a> = (String, &'a [String]);
pub type QueryParams = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct ItemWithDetails<ItemBase, ItemDetails> {
    pub base: ItemBase,
    pub details: ItemDetails,
}
