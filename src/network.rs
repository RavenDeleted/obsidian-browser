use reqwest::blocking::Client;
use url::Url;

pub fn init() {
    println!("[network] initialized");
}

pub fn fetch_url(url_str: &str) -> Result<String, String> {
    let url_str = if url_str.starts_with("http://") || url_str.starts_with("https://") {
        url_str.to_string()
    } else {
        format!("https://{}", url_str)
    };

    let url = Url::parse(&url_str)
        .map_err(|e| format!("URL invalide: {}", e))?;

    if url.scheme() != "https" {
        return Err("HTTPS uniquement".to_string());
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(false)
        .user_agent("ObsidianBrowser/0.1")
        .build()
        .map_err(|e| format!("Erreur client TLS: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Erreur réseau: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Erreur HTTP: {}", response.status()));
    }

    response
        .text()
        .map_err(|e| format!("Erreur lecture HTML: {}", e))
}
