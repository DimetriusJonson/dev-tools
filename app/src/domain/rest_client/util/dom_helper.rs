use dom_query::Document;
use html_cleaning::links::{is_absolute, resolve};

pub fn make_absolute_head_links(doc: &Document, base_url: &str) {
    for node in doc.select("link[href]").nodes() {
        let sel = dom_query::Selection::from(*node);
        if let Some(href) = sel.attr("href") {
            if !is_absolute(&href) {
                if let Some(absolute) = resolve(&href, base_url) {
                    sel.set_attr("href", &absolute);
                }
            }
        }
    }
}
