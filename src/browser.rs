use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    Box,
    Button,
    Entry,
    Orientation,
    ScrolledWindow,
    Notebook,
    Label,
    AboutDialog,
};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;

use webkit6::prelude::*;
use webkit6::{
    LoadEvent,
    WebView,
    PolicyDecisionType,
};

use url::Url;

use crate::privacy::PrivacyFilter;
use crate::network;
use crate::parser;
use crate::html;

// ✅ Page d'accueil : Instance SearXNG priv.au
const HOME: &str = "https://priv.au/";
const INVIDIOUS_INSTANCE: &str = "invidious.nerdvpn.de";

fn configure_privacy_settings(settings: &webkit6::Settings) {
    settings.set_enable_page_cache(false);
    settings.set_enable_developer_extras(false);
    settings.set_enable_javascript(true);
    settings.set_enable_webgl(false);
    settings.set_enable_media(false);
    settings.set_enable_webrtc(false);
    settings.set_user_agent(Some(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ));
}

fn youtube_to_invidious(url: &str) -> String {
    if let Ok(parsed) = Url::parse(url) {
        let path = parsed.path().to_string();
        let query = parsed.query().unwrap_or("");

        if path.starts_with("/watch") {
            if let Some(video_id) = parsed.query_pairs()
                .find(|(k, _)| k == "v")
                .map(|(_, v)| v.to_string())
            {
                return format!("https://{}/watch?v={}", INVIDIOUS_INSTANCE, video_id);
            }
        }
        if path.starts_with("/@") {
            let channel = path.trim_start_matches("/@");
            return format!("https://{}/@{}", INVIDIOUS_INSTANCE, channel);
        }
        if path.starts_with("/shorts/") {
            let short_id = path.trim_start_matches("/shorts/");
            return format!("https://{}/watch?v={}", INVIDIOUS_INSTANCE, short_id);
        }
        let mut invidious_url = format!("https://{}{}", INVIDIOUS_INSTANCE, path);
        if !query.is_empty() && !path.starts_with("/watch") {
            invidious_url = format!("{}?{}", invidious_url, query);
        }
        return invidious_url;
    }
    format!("https://{}/feed/trending", INVIDIOUS_INSTANCE)
}

pub fn start() {
    println!("[browser] activate");

    let app = Application::builder()
        .application_id("org.obsidian.browser")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Obsidian Browser 🔒")
            .default_width(1200)
            .default_height(800)
            .build();

        let root = Box::new(Orientation::Vertical, 5);
        let toolbar = Box::new(Orientation::Horizontal, 5);

        let back = Button::with_label("←");
        let forward = Button::with_label("→");
        let reload = Button::with_label("⟳");
        let home = Button::with_label("🏠");
        let fav_button = Button::with_label("⭐");
        let history_button = Button::with_label("📜");
        let reader_button = Button::with_label("📖");
        let about_button = Button::with_label("ℹ️");

        let entry = Entry::new();
        entry.set_hexpand(true);
        entry.set_placeholder_text(Some("🔍 Recherche ou URL..."));

        let go_button = Button::with_label("Go");
        let new_tab = Button::with_label("+");

        let notebook = Notebook::new();
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);
        notebook.set_scrollable(true);

        let privacy_filter = Rc::new(PrivacyFilter::new());
        let loading = Rc::new(RefCell::new(false));
        let history = Rc::new(RefCell::new(VecDeque::<String>::with_capacity(50)));
        let active_webview: Rc<RefCell<Option<WebView>>> = Rc::new(RefCell::new(None));

        let get_active_webview = {
            let active_webview = active_webview.clone();
            move || -> Option<WebView> {
                active_webview.borrow().clone()
            }
        };

        let create_tab_fn = {
            let notebook = notebook.clone();
            let loading = loading.clone();
            let entry = entry.clone();
            let privacy_filter = privacy_filter.clone();
            let history = history.clone();
            let active_webview = active_webview.clone();

            move || {
                let webview = WebView::new();
                println!("[webkit] nouvel onglet créé");

                webview.set_hexpand(true);
                webview.set_vexpand(true);

                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                    configure_privacy_settings(&settings);
                }

                let privacy_policy = privacy_filter.clone();
                webview.connect_decide_policy(move |_view, decision, decision_type| {
                    if decision_type == PolicyDecisionType::NavigationAction {
                        if let Some(nav_decision) =
                            decision.downcast_ref::<webkit6::NavigationPolicyDecision>()
                        {
                            if let Some(mut nav_action) = nav_decision.navigation_action() {
                                if let Some(request) = nav_action.request() {
                                    if let Some(uri) = request.uri() {
                                        let uri_str = uri.to_string();
                                        let uri_lower = uri_str.to_lowercase();

                                        if uri_lower.contains("youtube.com") || uri_lower.contains("youtu.be") {
                                            let invidious_url = youtube_to_invidious(&uri_str);
                                            println!("[redirection] YouTube → Invidious : {}", invidious_url);
                                            decision.ignore();
                                            let view = _view.clone();
                                            let url = invidious_url.clone();
                                            glib::timeout_add_local(
                                                std::time::Duration::from_millis(50),
                                                move || {
                                                    view.load_uri(&url);
                                                    glib::ControlFlow::Break
                                                }
                                            );
                                            return true;
                                        }

                                        if privacy_policy.check(&uri_str) {
                                            println!("[privacy] 🚫 navigation bloquée : {}", uri_str);
                                            decision.ignore();
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    false
                });

                let tab_box = Box::new(Orientation::Horizontal, 0);
                let tab_label = Label::new(Some("Nouvel onglet"));
                let close_button = Button::with_label("×");
                close_button.set_css_classes(&["circular"]);
                tab_box.append(&tab_label);
                tab_box.append(&close_button);

                let scrolled = ScrolledWindow::new();
                scrolled.set_child(Some(&webview));
                scrolled.set_vexpand(true);
                scrolled.set_hexpand(true);

                let page_index = notebook.append_page(&scrolled, Some(&tab_box));
                notebook.set_current_page(Some(page_index));

                *active_webview.borrow_mut() = Some(webview.clone());
                println!("[debug] active_webview mis à jour (onglet créé)");

                let notebook_clone = notebook.clone();
                let active_webview_clone = active_webview.clone();
                close_button.connect_clicked(move |_| {
                    if let Some(index) = notebook_clone.current_page() {
                        if notebook_clone.n_pages() > 1 {
                            notebook_clone.remove_page(Some(index));
                            if let Some(current) = notebook_clone.current_page() {
                                if let Some(page) = notebook_clone.nth_page(Some(current)) {
                                    if let Some(scrolled) = page.downcast_ref::<ScrolledWindow>() {
                                        if let Some(child) = scrolled.child() {
                                            if let Some(webview) = child.downcast_ref::<WebView>() {
                                                *active_webview_clone.borrow_mut() = Some(webview.clone());
                                            }
                                        }
                                    }
                                }
                            } else {
                                *active_webview_clone.borrow_mut() = None;
                            }
                        } else {
                            println!("[browser] impossible de fermer le dernier onglet");
                        }
                    }
                });

                let tab_label_clone = tab_label.clone();
                webview.connect_title_notify(move |view| {
                    if let Some(title) = view.title() {
                        tab_label_clone.set_text(&title);
                    }
                });

                let entry_clone = entry.clone();
                let notebook_clone = notebook.clone();
                let history_clone = history.clone();
                webview.connect_uri_notify(move |view| {
                    if let Some(index) = notebook_clone.current_page() {
                        if let Some(page) = notebook_clone.nth_page(Some(index)) {
                            if let Some(scrolled) = page.downcast_ref::<ScrolledWindow>() {
                                if let Some(child) = scrolled.child() {
                                    if let Some(webview) = child.downcast_ref::<WebView>() {
                                        if webview == view {
                                            if let Some(uri) = view.uri() {
                                                let uri_str = uri.to_string();
                                                entry_clone.set_text(&uri_str);
                                                let mut history = history_clone.borrow_mut();
                                                if history.back() != Some(&uri_str) {
                                                    history.push_back(uri_str);
                                                    if history.len() > 50 {
                                                        history.pop_front();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });

                let loading_state = loading.clone();
                webview.connect_load_changed(move |_view, event| {
                    println!("[webkit] {:?}", event);
                    if event == LoadEvent::Finished {
                        *loading_state.borrow_mut() = false;
                    }
                });

                let loading_failed = loading.clone();
                webview.connect_load_failed(move |_view, event, uri, error| {
                    println!("[webkit ERROR] {:?} {} {}", event, uri, error);
                    *loading_failed.borrow_mut() = false;
                    false
                });

                webview.load_uri(HOME);
                webview
            }
        };

        let _first_webview = create_tab_fn();

        let entry_switch = entry.clone();
        let notebook_switch = notebook.clone();
        let active_webview_switch = active_webview.clone();
        notebook_switch.clone().connect_switch_page(move |_notebook, _page, _page_num| {
            if let Some(index) = notebook_switch.current_page() {
                if let Some(page) = notebook_switch.nth_page(Some(index)) {
                    if let Some(scrolled) = page.downcast_ref::<ScrolledWindow>() {
                        if let Some(child) = scrolled.child() {
                            if let Some(webview) = child.downcast_ref::<WebView>() {
                                *active_webview_switch.borrow_mut() = Some(webview.clone());
                                if let Some(uri) = webview.uri() {
                                    entry_switch.set_text(&uri.to_string());
                                }
                            }
                        }
                    }
                }
            }
        });

        let create_tab_fn_clone = create_tab_fn.clone();
        let action_new_tab = gtk4::gio::SimpleAction::new("new-tab", None);
        action_new_tab.connect_activate(move |_, _| {
            create_tab_fn_clone();
        });
        window.add_action(&action_new_tab);

        let notebook_clone = notebook.clone();
        let action_close_tab = gtk4::gio::SimpleAction::new("close-tab", None);
        action_close_tab.connect_activate(move |_, _| {
            if let Some(index) = notebook_clone.current_page() {
                if notebook_clone.n_pages() > 1 {
                    notebook_clone.remove_page(Some(index));
                }
            }
        });
        window.add_action(&action_close_tab);

        let entry_clone = entry.clone();
        let action_focus_url = gtk4::gio::SimpleAction::new("focus-url", None);
        action_focus_url.connect_activate(move |_, _| {
            entry_clone.grab_focus();
            entry_clone.select_region(0, -1);
        });
        window.add_action(&action_focus_url);

        let notebook_clone = notebook.clone();
        let action_reload = gtk4::gio::SimpleAction::new("reload", None);
        action_reload.connect_activate(move |_, _| {
            if let Some(index) = notebook_clone.current_page() {
                if let Some(page) = notebook_clone.nth_page(Some(index)) {
                    if let Some(scrolled) = page.downcast_ref::<ScrolledWindow>() {
                        if let Some(child) = scrolled.child() {
                            if let Some(webview) = child.downcast_ref::<WebView>() {
                                webview.reload();
                            }
                        }
                    }
                }
            }
        });
        window.add_action(&action_reload);

        let history_clone = history.clone();
        let action_history = gtk4::gio::SimpleAction::new("show-history", None);
        action_history.connect_activate(move |_, _| {
            println!("[historique] 📜 50 dernières URLs :");
            for (i, url) in history_clone.borrow().iter().enumerate() {
                println!("  {}: {}", i+1, url);
            }
        });
        window.add_action(&action_history);

        let app_clone = app.clone();
        let action_quit = gtk4::gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(move |_, _| {
            app_clone.quit();
        });
        window.add_action(&action_quit);

        app.set_accels_for_action("win.new-tab", &["<Ctrl>T"]);
        app.set_accels_for_action("win.close-tab", &["<Ctrl>W"]);
        app.set_accels_for_action("win.focus-url", &["<Ctrl>L"]);
        app.set_accels_for_action("win.reload", &["<Ctrl>R"]);
        app.set_accels_for_action("win.show-history", &["<Ctrl><Shift>H"]);
        app.set_accels_for_action("win.quit", &["<Ctrl>Q"]);

        let create_tab_fn_clone = create_tab_fn.clone();
        new_tab.connect_clicked(move |_| {
            create_tab_fn_clone();
        });

        back.connect_clicked({
            let get = get_active_webview.clone();
            move |_| {
                if let Some(v) = get() {
                    if v.can_go_back() { v.go_back(); }
                }
            }
        });

        forward.connect_clicked({
            let get = get_active_webview.clone();
            move |_| {
                if let Some(v) = get() {
                    if v.can_go_forward() { v.go_forward(); }
                }
            }
        });

        reload.connect_clicked({
            let get = get_active_webview.clone();
            move |_| {
                if let Some(v) = get() { v.reload(); }
            }
        });

        home.connect_clicked({
            let get = get_active_webview.clone();
            move |_| {
                if let Some(v) = get() { v.load_uri(HOME); }
            }
        });

        fav_button.connect_clicked({
            let get = get_active_webview.clone();
            move |_| {
                if let Some(webview) = get() {
                    if let Some(uri) = webview.uri() {
                        let uri_str = uri.to_string();
                        let mut file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("favorites.txt")
                            .unwrap_or_else(|_| {
                                OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .open("favorites.txt")
                                    .unwrap()
                            });
                        writeln!(file, "{}", uri_str).unwrap();
                        println!("[favori] ⭐ ajouté : {}", uri_str);
                    }
                }
            }
        });

        history_button.connect_clicked({
            let history_clone = history.clone();
            move |_| {
                println!("[historique] 📜 50 dernières URLs :");
                for (i, url) in history_clone.borrow().iter().enumerate() {
                    println!("  {}: {}", i+1, url);
                }
            }
        });

        reader_button.connect_clicked({
            let get = get_active_webview.clone();
            let notebook = notebook.clone();
            let loading = loading.clone();

            move |_| {
                if let Some(webview) = get() {
                    if let Some(uri) = webview.uri() {
                        let url = uri.to_string();
                        println!("[reader] 📖 Chargement de {}", url);
                        *loading.borrow_mut() = true;

                        match network::fetch_url(&url) {
                            Ok(html_content) => {
                                let parsed = parser::parse(&html_content);
                                let clean = html::render(&html_content);

                                let reader_view = WebView::new();
                                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&reader_view) {
                                    settings.set_enable_javascript(false);
                                    settings.set_enable_webgl(false);
                                    settings.set_enable_media(false);
                                    settings.set_user_agent(Some("Mozilla/5.0 ObsidianReader"));
                                }

                                let content = format!(
                                    r#"
                                    <html>
                                    <head>
                                        <meta charset="utf-8">
                                        <title>{}</title>
                                        <style>
                                            body {{ background: #f9f9f9; color: #222; font-family: Georgia, serif; max-width: 800px; margin: 40px auto; padding: 20px; line-height: 1.6; }}
                                            h1 {{ font-size: 2em; border-bottom: 1px solid #ccc; }}
                                            pre {{ white-space: pre-wrap; font-family: Georgia, serif; font-size: 1.1em; }}
                                            .meta {{ color: #666; font-size: 0.9em; font-family: sans-serif; }}
                                        </style>
                                    </head>
                                    <body>
                                        <h1>{}</h1>
                                        <p class="meta">
                                            <strong>Analyse Obsidian :</strong> {} liens détectés | {} caractères de texte extraits
                                        </p>
                                        <hr>
                                        <pre>{}</pre>
                                        <hr>
                                        <p class="meta">Mode lecture - Obsidian Browser</p>
                                    </body>
                                    </html>
                                    "#,
                                    parsed.title,
                                    parsed.title,
                                    parsed.links.len(),
                                    parsed.text.len(),
                                    clean
                                );

                                reader_view.load_html(&content, Some(&url));

                                let scrolled = ScrolledWindow::new();
                                scrolled.set_child(Some(&reader_view));
                                scrolled.set_vexpand(true);
                                scrolled.set_hexpand(true);

                                let tab_label = Label::new(Some(&format!("📖 {}", parsed.title)));
                                let tab_box = Box::new(Orientation::Horizontal, 0);
                                tab_box.append(&tab_label);
                                let close_button = Button::with_label("×");
                                close_button.set_css_classes(&["circular"]);
                                tab_box.append(&close_button);

                                let page_index = notebook.append_page(&scrolled, Some(&tab_box));
                                notebook.set_current_page(Some(page_index));

                                let notebook_clone = notebook.clone();
                                close_button.connect_clicked(move |_| {
                                    if let Some(idx) = notebook_clone.current_page() {
                                        notebook_clone.remove_page(Some(idx));
                                    }
                                });

                                *loading.borrow_mut() = false;
                                println!("[reader] ✅ Affichage du mode lecture");
                            }
                            Err(e) => {
                                println!("[reader] ❌ Erreur : {}", e);
                                *loading.borrow_mut() = false;
                            }
                        }
                    }
                }
            }
        });

        about_button.connect_clicked(move |_| {
            let about = AboutDialog::builder()
                .program_name("Obsidian Browser")
                .version("0.1.0")
                .comments("Navigateur web minimal, sécurisé et respectueux de la vie privée")
                .license_type(gtk4::License::MitX11)
                .build();
            about.present();
        });

        let navigate_fn = {
            let entry = entry.clone();
            let loading = loading.clone();
            let privacy_filter = privacy_filter.clone();
            let get_active_webview = get_active_webview.clone();

            move || {
                if *loading.borrow() { return; }

                let mut url = entry.text().to_string();
                if url.is_empty() { return; }

                let url_lower = url.to_lowercase();
                if url_lower.contains("youtube.com") || url_lower.contains("youtu.be") {
                    url = youtube_to_invidious(&url);
                }

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    if url.contains(' ') || !url.contains('.') {
                        // ✅ Recherche avec l'instance SearXNG priv.au
                        url = format!("https://priv.au/search?q={}", url.replace(' ', "+"));
                    } else {
                        url = format!("https://{}", url);
                    }
                }

                if privacy_filter.check(&url) { return; }

                if let Some(webview) = get_active_webview() {
                    *loading.borrow_mut() = true;
                    webview.load_uri(&url);
                }
            }
        };

        go_button.connect_clicked({
            let nav = navigate_fn.clone();
            move |_| { nav(); }
        });

        entry.connect_activate({
            let nav = navigate_fn.clone();
            move |_| { nav(); }
        });

        toolbar.append(&back);
        toolbar.append(&forward);
        toolbar.append(&reload);
        toolbar.append(&home);
        toolbar.append(&fav_button);
        toolbar.append(&history_button);
        toolbar.append(&reader_button);
        toolbar.append(&about_button);
        toolbar.append(&entry);
        toolbar.append(&go_button);
        toolbar.append(&new_tab);

        root.append(&toolbar);
        root.append(&notebook);

        window.set_child(Some(&root));
        window.present();

        println!("[browser] 🔒 ready - mode vie privée extrême activé");
    });

    app.run();
}
