use super::*;

use reqwest::Url;

type Data = scraper::Html;

fn converter(raw: &str) -> Result<Data> {
    Ok(scraper::Html::parse_document(raw))
}

pub struct HtmlScraper<ItemBase, ItemDetails> {
    pub inner: WebScraper<Data, ItemBase, ItemDetails>,
}

impl<ItemBase: Debug + Clone, ItemDetails: Debug + Clone> HtmlScraper<ItemBase, ItemDetails> {
    pub fn new(base_url: &Url, list_path: &str, mock: bool, base_parser: BaseParser<Data, ItemBase>, details_parser: DetailsParser<Data, ItemDetails>) -> Self {
        let inner = WebScraper::new(base_url, list_path, mock, converter, base_parser, details_parser);
        HtmlScraper { inner }
    }
}
