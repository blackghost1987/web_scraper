use super::*;

type Data = serde_json::Value;

fn converter(raw: &str) -> Result<Data> {
    serde_json::from_str(raw).map_err(Error::from)
}

pub struct JsonScraper<ItemBase, ItemDetails> {
    pub inner: WebScraper<Data, ItemBase, ItemDetails>,
}

impl<ItemBase: Debug + Clone, ItemDetails: Debug + Clone> JsonScraper<ItemBase, ItemDetails> {
    pub fn new(base_url: &Url, list_path: &str, mock: bool, base_parser: BaseParser<Data, ItemBase>, details_parser: DetailsParser<Data, ItemDetails>) -> Self {
        let inner = WebScraper::new(base_url, list_path, mock, converter, base_parser, details_parser);
        JsonScraper { inner }
    }
}

pub trait JsonParser {
    fn parse(value: &Data) -> Result<Self>
    where
        Self: Sized;
}
