use super::*;

use reqwest::{StatusCode, Url};
use std::fmt::Debug;
use std::{fs, thread, time};

mod html_scraper;
pub use html_scraper::*;

mod json_scraper;
pub use json_scraper::*;

pub struct PaginationData {
    pub current_page: u16,
    pub total_pages: Option<u16>,
    pub next_page_query: Option<QueryParam>,
}

type Converter<Data> = fn(&str) -> Result<Data>;
type ListParser<Data, ItemBase> = fn(&Url, Data) -> Result<(Vec<ItemBase>, Option<PaginationData>)>;
type DetailsParser<Data, ItemDetails> = fn(Data) -> Result<ItemDetails>;

pub struct WebScraper<Data, ItemBase, ItemDetails> {
    client: reqwest::blocking::Client,
    base_url: Url,
    list_path: String,
    mock: bool,
    converter: Converter<Data>,
    list_parser: ListParser<Data, ItemBase>,
    details_parser: DetailsParser<Data, ItemDetails>,
    delay: Option<time::Duration>,
}

impl<Data, ItemBase: Debug + Clone, ItemDetails: Debug + Clone> WebScraper<Data, ItemBase, ItemDetails> {
    pub fn new(
        base_url: &Url,
        list_path: &str,
        mock: bool,
        converter: Converter<Data>,
        base_parser: ListParser<Data, ItemBase>,
        details_parser: DetailsParser<Data, ItemDetails>,
        delay: Option<time::Duration>,
    ) -> Self {
        let client = reqwest::blocking::Client::builder().build().expect("client should be created");
        WebScraper {
            client,
            base_url: base_url.to_owned(),
            list_path: list_path.to_owned(),
            mock,
            converter,
            list_parser: base_parser,
            details_parser,
            delay,
        }
    }

    fn get_data(&self, url: &Url, query_opt: Option<&QueryParams>) -> Result<String> {
        let mut request = self.client.get(url.to_owned());
        if let Some(query) = query_opt {
            request = request.query(query);
        }
        let response = request.send()?;

        match response.status() {
            StatusCode::OK => response.text().map_err(From::from),
            StatusCode::TOO_MANY_REQUESTS => Err(WebScraperError("Rate limited".to_string())),
            other => Err(WebScraperError(format!("Unexpected error code: {}", other))),
        }
    }

    fn get_items_list_page(&self, query: &QueryParams) -> Result<Data> {
        let list_raw = if self.mock {
            let contents = fs::read_to_string("example/results.html").expect("mock list file reading failed");
            contents
        } else {
            let url = self.base_url.join(&self.list_path).expect("URL join should work");
            println!("Getting items list from {} with params: {:?}...", url, query);
            self.get_data(&url, Some(query))?
        };
        (self.converter)(&list_raw)
    }

    pub fn get_items_list(&self, query: &QueryParams) -> Result<Vec<ItemBase>> {
        let mut items: Vec<ItemBase> = vec![];
        let mut finished = false;
        let mut page_query = query.clone();
        let mut page_counter = 0;

        while !finished {
            let document = self.get_items_list_page(&page_query)?;
            let (mut page_items, pagination_opt) = (self.list_parser)(&self.base_url, document)?;
            page_counter = page_counter + 1;

            items.append(&mut page_items);

            finished = match pagination_opt {
                Some(pagination) => {
                    println!("Got page {} of {:?}, next page query: {:?}", pagination.current_page, pagination.total_pages, pagination.next_page_query);

                    match pagination.next_page_query {
                        Some(next_query) => {
                            let mut next_page_query = page_query.clone();
                            next_page_query.insert(next_query.0, next_query.1);

                            //println!("Would ask next_page_query to be: {:?}", next_page_query);

                            if next_page_query == page_query {
                                println!("Query is the same as the last one, stopping to prevent infinite loop");
                                true
                            } else {
                                // let's get the next page
                                page_query = next_page_query;
                                if let Some(duration) = self.delay {
                                    thread::sleep(duration);
                                }
                                false
                            }
                        },
                        None => {
                            println!("Pagination finished, got {} pages", page_counter);
                            true
                        },
                    }
                },
                None => {
                    println!("No pagination, got 1 page");
                    true
                },
            };
        }

        let total_count = items.len();

        // TODO move this to GsmarenaScraper
        if total_count == 70 {
            panic!("Exactly 70 items in list! Check if your search query is refined enough!");
        } else {
            println!("Found {} items in {} pages", total_count, page_counter)
        }

        Ok(items)
    }

    pub fn get_items_split_by(&self, query: QueryParams, splitting: QuerySplitter) -> Result<Vec<ItemBase>> {
        let mut items: Vec<ItemBase> = vec![];

        let (split_by, values) = splitting;

        for v in values {
            let mut query_local = query.clone();
            query_local.insert(split_by.clone(), v.to_owned());
            println!("Query page: {:?}", query_local);
            let mut items_list = self.get_items_list(&query_local)?;
            items.append(&mut items_list);
        }

        Ok(items)
    }

    fn get_item_details(&self, url: &Url) -> Result<Data> {
        let details_raw = if self.mock {
            let contents = fs::read_to_string("example/device_details.html").expect("mock details file reading failed");
            contents
        } else {
            if let Some(duration) = self.delay {
                thread::sleep(duration);
            }
            self.get_data(url, None)?
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
