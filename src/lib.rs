pub mod error;
use crate::error::*;

use reqwest::{StatusCode, Url};
use scraper::Html;
use std::fmt::Debug;
use std::{fs, thread, time};

pub type QueryParam = (&'static str, &'static str);
pub type QuerySplitter<'a> = (&'static str, &'a [&'static str]);
pub type QueryParams<'a> = &'a [QueryParam];

#[derive(Debug, Clone)]
pub struct ItemWithDetails<ItemBase, ItemDetails> {
    pub base: ItemBase,
    pub details: ItemDetails,
}

pub type BaseParser<ItemBase> = fn(&Url, Html) -> Result<Vec<ItemBase>>;
pub type DetailsParser<ItemDetails> = fn(String) -> Result<ItemDetails>;

pub struct ScraperClient<ItemBase, ItemDetails> {
    pub client: reqwest::blocking::Client,
    pub base_url: Url,
    pub list_path: String,
    pub mock: bool,
    pub base_parser: BaseParser<ItemBase>,
    pub details_parser: DetailsParser<ItemDetails>,
}

impl<ItemBase: Debug + Clone, ItemDetails: Debug + Clone> ScraperClient<ItemBase, ItemDetails> {
    pub fn new(base_url: &Url, list_path: &str, mock: bool, base_parser: BaseParser<ItemBase>, details_parser: DetailsParser<ItemDetails>) -> Self {
        let client = reqwest::blocking::Client::builder().build().expect("client should be created");

        ScraperClient {
            client,
            base_url: base_url.to_owned(),
            list_path: list_path.to_owned(),
            mock,
            base_parser,
            details_parser,
        }
    }

    fn get_html(&self, url: &Url, query: QueryParams) -> Result<String> {
        let response = self.client.get(url.to_owned()).query(query).send()?;

        match response.status() {
            StatusCode::OK => response.text().map_err(From::from),
            StatusCode::TOO_MANY_REQUESTS => Err(WebScraperError("Rate limited".to_string())),
            other => Err(WebScraperError(format!("Unexpected error code: {}", other))),
        }
    }

    fn get_items_list(&self, query: QueryParams) -> Result<String> {
        if self.mock {
            let contents = fs::read_to_string("example/results.html").expect("mock list file reading failed");
            Ok(contents)
        } else {
            println!("Getting items list from {}...", self.base_url);
            let url = self.base_url.join(&self.list_path).expect("URL join should work");
            self.get_html(&url, query)
        }
    }

    pub fn get_items_base_data(&self, query: QueryParams) -> Result<Vec<ItemBase>> {
        let list_html = self.get_items_list(query)?;
        let document = Html::parse_document(&list_html);

        let base_data: Vec<ItemBase> = (self.base_parser)(&self.base_url, document)?;

        let item_count = base_data.len();

        if item_count == 70 {
            panic!("Exactly 70 items in list! Check if your search query is refined enough!");
        } else {
            println!("Found {} devices", item_count)
        }

        Ok(base_data)
    }

    pub fn get_items_split_by(&self, query: QueryParams, splitting: QuerySplitter) -> Result<Vec<ItemBase>> {
        let mut items: Vec<ItemBase> = vec![];

        let (split_by, values) = splitting;

        for v in values {
            let mut query_local = query.to_vec();
            let splitting_filter: QueryParam = (split_by, v);
            query_local.push(splitting_filter);
            println!("Query page: {:?}", query_local);
            let mut items_page = self.get_items_base_data(&query_local)?;
            items.append(&mut items_page);
        }

        Ok(items)
    }

    fn get_item_details(&self, url: &Url) -> Result<String> {
        if self.mock {
            let contents = fs::read_to_string("example/device_details.html").expect("mock details file reading failed");
            Ok(contents)
        } else {
            thread::sleep(time::Duration::from_secs(5));
            self.get_html(url, &[])
        }
    }

    pub fn get_and_parse_item_details(&self, url: &Url, base: &ItemBase) -> Result<ItemWithDetails<ItemBase, ItemDetails>> {
        println!("Downloading and parsing item details: {:?} - {:?}", url, base);
        let doc = self.get_item_details(url)?;

        let details = (self.details_parser)(doc)?;
        println!("\tItem details: {:?}", details);

        Ok(ItemWithDetails {
            base: base.to_owned(),
            details,
        })
    }
}
