use crate::error::*;

pub mod error;
pub mod scrapers;

pub type QueryParam = (&'static str, &'static str);
pub type QuerySplitter<'a> = (&'static str, &'a [&'static str]);
pub type QueryParams<'a> = &'a [QueryParam];

#[derive(Debug, Clone)]
pub struct ItemWithDetails<ItemBase, ItemDetails> {
    pub base: ItemBase,
    pub details: ItemDetails,
}
