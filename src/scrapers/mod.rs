use super::*;

use reqwest::{StatusCode, Url};
use std::fmt::Debug;
use std::{fs, thread, time};

mod html_scraper;
pub use html_scraper::*;

mod json_scraper;
pub use json_scraper::*;

type Converter<Data> = fn(&str) -> Result<Data>;
type BaseParser<Data, ItemBase> = fn(&Url, Data) -> Result<Vec<ItemBase>>;
type DetailsParser<Data, ItemDetails> = fn(Data) -> Result<ItemDetails>;

pub struct WebScraper<Data, ItemBase, ItemDetails> {
    client: reqwest::blocking::Client,
    base_url: Url,
    list_path: String,
    mock: bool,
    converter: Converter<Data>,
    base_parser: BaseParser<Data, ItemBase>,
    details_parser: DetailsParser<Data, ItemDetails>,
}

impl<Data, ItemBase: Debug + Clone, ItemDetails: Debug + Clone> WebScraper<Data, ItemBase, ItemDetails> {
    pub fn new(
        base_url: &Url,
        list_path: &str,
        mock: bool,
        converter: Converter<Data>,
        base_parser: BaseParser<Data, ItemBase>,
        details_parser: DetailsParser<Data, ItemDetails>,
    ) -> Self {
        let client = reqwest::blocking::Client::builder().build().expect("client should be created");
        WebScraper {
            client,
            base_url: base_url.to_owned(),
            list_path: list_path.to_owned(),
            mock,
            converter,
            base_parser,
            details_parser,
        }
    }

    fn get_data(&self, url: &Url, query: QueryParams) -> Result<String> {
        let response = self.client.get(url.to_owned()).query(query).send()?;

        match response.status() {
            StatusCode::OK => response.text().map_err(From::from),
            StatusCode::TOO_MANY_REQUESTS => Err(WebScraperError("Rate limited".to_string())),
            other => Err(WebScraperError(format!("Unexpected error code: {}", other))),
        }
    }

    fn get_items_list(&self, query: QueryParams) -> Result<Data> {
        let list_raw = if self.mock {
            let contents = fs::read_to_string("example/results.html").expect("mock list file reading failed");
            contents
        } else {
            let url = self.base_url.join(&self.list_path).expect("URL join should work");
            println!("Getting items list from {}...", url);
            self.get_data(&url, query)?
        };
        (self.converter)(&list_raw)
    }

    pub fn get_items_base_data(&self, query: QueryParams) -> Result<Vec<ItemBase>> {
        let document = self.get_items_list(query)?;

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

    fn get_item_details(&self, url: &Url) -> Result<Data> {
        let details_raw = if self.mock {
            let contents = fs::read_to_string("example/device_details.html").expect("mock details file reading failed");
            contents
        } else {
            thread::sleep(time::Duration::from_secs(5));
            self.get_data(url, &[])?
        };
        (self.converter)(&details_raw)
    }

    pub fn get_and_parse_item_details(&self, url: &Url, base: &ItemBase) -> Result<ItemWithDetails<ItemBase, ItemDetails>> {
        println!("Downloading and parsing item details: {} - {:?}", url, base);
        let doc = self.get_item_details(url)?;

        let details = (self.details_parser)(doc)?;
        println!("\tItem details: {:?}", details);

        Ok(ItemWithDetails { base: base.clone(), details })
    }
}

pub trait TextParser {
    fn parse(text: &str) -> Result<Self>
    where
        Self: Sized;
}
