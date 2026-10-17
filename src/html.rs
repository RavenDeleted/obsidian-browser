use html5ever::parse_document;
use html5ever::tendril::TendrilSink;

use markup5ever_rcdom::{
    Handle,
    NodeData,
    RcDom,
};

pub fn render(html: &str) -> String {
    let dom = parse_document(
        RcDom::default(),
        Default::default()
    ).one(html);

    let mut output = String::new();
    walk(&dom.document, &mut output, false);
    cleanup(&output)
}

fn walk(
    handle: &Handle,
    output: &mut String,
    blocked: bool
) {
    let mut blocked_here = blocked;

    match &handle.data {
        NodeData::Element { name, .. } => {
            let tag = name.local.to_string().to_lowercase();

            match tag.as_str() {
                "script" | "style" | "head" | "meta" | "link" |
                "svg" | "noscript" | "nav" | "footer" | "header" |
                "form" | "iframe" => {
                    blocked_here = true;
                }
                "main" | "article" | "section" => {
                    if !blocked { output.push('\n'); }
                }
                "h1" | "h2" | "h3" => {
                    if !blocked { output.push_str("\n\n"); }
                }
                "p" => {
                    if !blocked { output.push('\n'); }
                }
                "li" => {
                    if !blocked { output.push_str("\n- "); }
                }
                "br" => {
                    if !blocked { output.push('\n'); }
                }
                _ => {}
            }
        }
        NodeData::Text { contents } => {
            if !blocked {
                let text = contents.borrow();
                let clean = text.trim();
                if !clean.is_empty() {
                    output.push_str(clean);
                    output.push(' ');
                }
            }
        }
        _ => {}
    }

    for child in handle.children.borrow().iter() {
        walk(child, output, blocked_here);
    }
}

fn cleanup(input: &str) -> String {
    let mut result = String::new();
    let mut empty_lines = 0;
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            empty_lines += 1;
            if empty_lines <= 1 { result.push('\n'); }
            continue;
        }
        empty_lines = 0;
        result.push_str(line);
        result.push('\n');
    }
    result
}
