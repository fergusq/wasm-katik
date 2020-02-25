#![feature(map_first_last)]

mod utils;

extern crate klingon_utils;
use klingon_utils::zrajm::ZrajmDictionary;
use klingon_utils::morpho::completions;

use wasm_bindgen::prelude::*;

use std::cell::RefCell;

thread_local! {
    static DICT: RefCell<Option<ZrajmDictionary>> = RefCell::new(None);
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen(start)]
pub fn load_dictionary() {
    utils::set_panic_hook();
    DICT.with(|dict| {
        let mut dictionary = ZrajmDictionary::parse(
            String::from_utf8(include_bytes!("../dict.zdb").into_iter().cloned().collect()).unwrap().lines().map(|l| l.to_string())
        );
        dictionary.words.retain(|w| w.id != "UjQ");
        for (_, m) in &mut dictionary.pos_index {
            m.retain(|w| w.id != "UjQ");
        }
        *dict.borrow_mut() = Some(dictionary);
    });
}

#[wasm_bindgen]
pub fn get_completions(text: String, lang: String) -> String {
    let words: Vec<_> = text.split(" ").collect();
    let last_word = *words.last().unwrap_or(&"");
    DICT.with(|dict: &RefCell<Option<ZrajmDictionary>>| {
        let dict = dict.borrow();
        let ref dict = *dict;
        if let Some(ref dict) = dict {
            let compls = completions(dict, last_word);
            let mut result = String::new();

            for parse in compls.parsed {
                result += format!("<b>{}:</b>", if lang == "sv" { "Analys" } else { "Analysis" }).as_str();
                result += "<dl>";
                for word in parse {
                    result += format!("<dh>{}</dh>", escape_html(word.first().unwrap().tlh.clone())).as_str();
                    for homonym in word {
                        result += format!("<dd>{}</dd>",
                            escape_html(remove_quotes(if lang == "sv" { homonym.sv } else { homonym.en }.join(", ")))
                        ).as_str();
                    }
                }
                result += "</dl>";
                result += "<hr>";
            }
            
            result += "<table>";
            for suggestion in compls.suggestions {
                result += format!("<tr><td><b>{}</b></td><td>{}</td></tr>",
                    escape_html(suggestion.tlh),
                    escape_html(remove_quotes(if lang == "sv" { suggestion.sv } else { suggestion.en }.join(", ")))
                ).as_str();
            }
            result += "</table>";
            result
        } else {
            "".to_string()
        }
        
    })
}

fn escape_html(text: String) -> String {
    text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
}

fn remove_quotes(text: String) -> String {
    text.replace("<", "").replace(">", "").replace("«", "").replace("»", "")
}