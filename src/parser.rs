use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub text: String,
    pub links: Vec<String>,
}

pub fn parse(html: &str) -> Page {
    let document = Html::parse_document(html);
    let title = extract_title(&document);
    let text = extract_content(&document);
    let links = extract_links(&document);
    Page { title, text, links }
}

fn extract_title(document: &Html) -> String {
    let selector = Selector::parse("title").unwrap();
    match document.select(&selector).next() {
        Some(element) => clean(
            &element.text().collect::<Vec<_>>().join(" ")
        ),
        None => String::new(),
    }
}

fn extract_content(document: &Html) -> String {
    let selector = Selector::parse("main article p h1 h2 h3 li").unwrap();
    let mut result = String::new();
    let mut previous = String::new();

    for element in document.select(&selector) {
        let text = clean(
            &element.text().collect::<Vec<_>>().join(" ")
        );
        if text.is_empty() { continue; }
        if text == previous { continue; }
        previous = text.clone();
        result.push_str(&text);
        result.push_str("\n\n");
    }

    result
}

fn extract_links(document: &Html) -> Vec<String> {
    let selector = Selector::parse("a[href]").unwrap();
    let mut links = Vec::new();
    for element in document.select(&selector) {
        if let Some(url) = element.value().attr("href") {
            if url.starts_with("http") && !links.contains(&url.to_string()) {
                links.push(url.to_string());
            }
        }
    }
    links
}

fn clean(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
