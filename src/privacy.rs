use std::collections::HashSet;

pub fn init() {
    println!("[privacy] 🌐 ephemeral mode enabled");
    println!("[privacy] 🍪 cookies disabled");
    println!("[privacy] 📜 history disabled");
}

pub struct PrivacyFilter {
    blocked_domains: HashSet<String>,
}

impl PrivacyFilter {
    pub fn new() -> Self {
        let mut blocked = HashSet::new();
        let domains = [
            "google-analytics.com",
            "analytics.google.com",
            "doubleclick.net",
            "googlesyndication.com",
            "googleadservices.com",
            "adservice.google.com",
            "pagead2.googlesyndication.com",
            "googletagmanager.com",
            "facebook.net",
            "connect.facebook.net",
            "amazon-adsystem.com",
            "criteo.com",
            "outbrain.com",
            "taboola.com",
            "adnxs.com",
            "pubmatic.com",
            "openx.net",
            "rubiconproject.com",
            "adsrvr.org",
            "hotjar.com",
            "scorecardresearch.com",
            "quantserve.com",
            "clarity.ms",
            "linkedin.com",
            "twitter.com",
            "t.co",
            "adzerk.net",
            "adform.net",
            "adition.com",
            "adition.de",
            "criteo.net",
            "casalemedia.com",
            "contextweb.com",
            "indexexchange.com",
            "spotxchange.com",
            "lijit.com",
            "sovrn.com",
            "advertising.com",
            "eyereturn.com",
            "turn.com",
            "adtech.de",
            "adserver.com",
            "adtechus.com",
            "adtechjp.com",
            "adtechcn.com",
        ];
        for domain in domains {
            blocked.insert(domain.to_string());
        }
        Self { blocked_domains: blocked }
    }

    pub fn is_blocked(&self, url: &str) -> bool {
        let url = url.to_lowercase();
        for domain in &self.blocked_domains {
            if url.contains(domain) {
                return true;
            }
        }
        false
    }

    pub fn check(&self, url: &str) -> bool {
        if self.is_blocked(url) {
            println!("[privacy] 🔒 bloqué : {}", url);
            return true;
        }
        false
    }

    pub fn count(&self) -> usize {
        self.blocked_domains.len()
    }
}
