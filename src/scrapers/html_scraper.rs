use super::*;

use reqwest::Url;
use scraper::{ElementRef, Selector};

type Data = scraper::Html;

fn converter(raw: &str) -> Result<Data> {
    Ok(scraper::Html::parse_document(raw))
}

pub struct HtmlScraper<ItemBase, ItemDetails> {
    pub inner: WebScraper<Data, ItemBase, ItemDetails>,
}

impl<ItemBase: Display + Clone, ItemDetails: Debug + Clone> HtmlScraper<ItemBase, ItemDetails> {
    pub fn new(
        base_url: &Url,
        list_path: &str,
        mock: bool,
        base_parser: ListParser<Data, ItemBase>,
        details_parser: DetailsParser<Data, ItemDetails>,
        delay: Option<time::Duration>,
    ) -> Self {
        let inner = WebScraper::new(base_url, list_path, mock, converter, base_parser, details_parser, delay);
        HtmlScraper { inner }
    }
}

pub trait HtmlParser {
    fn parse(element: &ElementRef) -> Result<Self>
    where
        Self: Sized;
}

pub fn get_first_text_by_selector(element: &ElementRef, selector: &Selector) -> Option<String> {
    let first_matching = element.select(selector).next()?;
    //debug!("First matching: {:?}", first_matching);
    let text = get_first_text_part(&first_matching)?;
    Some(text)
}

pub fn get_text_by_selector(element: &ElementRef, selector: &Selector) -> Option<String> {
    let first_matching = element.select(selector).next()?;
    let text = get_all_text(&first_matching)?;
    Some(text)
}

pub fn get_first_text_part(element: &ElementRef) -> Option<String> {
    let parts = element.text().collect::<Vec<_>>();
    let text = parts.first()?;
    Some(text.trim().to_string())
}

pub fn get_all_text(element: &ElementRef) -> Option<String> {
    let parts = element.text().collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    let trimmed: Vec<&str> = parts.iter().map(|s| s.trim()).collect();
    let joined = trimmed.join(" ");
    Some(joined)
}
