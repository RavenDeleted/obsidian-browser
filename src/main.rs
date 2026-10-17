mod browser;
mod network;
mod html;
mod parser;
mod privacy;
mod sandbox;

use std::env;

fn main() {
    // Variables d'environnement pour WebKit
    env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    env::set_var("GDK_BACKEND", "x11");
    env::set_var("WEBKIT_DNS_OVER_HTTPS", "https://1.1.1.1/dns-query");

    println!("🦉 Obsidian Browser - Vie privée extrême");

    // 🛡️ Activation du sandbox C (résout le warning de sandbox.rs)
    sandbox::enable();

    // 🛡️ Initialisation de la vie privée
    privacy::init();
    let filter = privacy::PrivacyFilter::new();
    println!("[privacy] 🛡️ {} domaines de tracking bloqués", filter.count());

    // 🌐 Initialisation du réseau
    network::init();

    // 🚀 Lancement du navigateur
    browser::start();
}
