// https://docs.rs/scraper/latest/scraper/
// https://docs.rs/ureq/latest/ureq/
use ureq;
use scraper::{Html, ElementRef, Selector};

const FIRST_CC_NAME: &str = "Daron Nefcy";

fn get_first_index_node(raw_html: String) -> ElementRef {
    let parsed_html: Html = Html::parse_document(&raw_html);
    let span_selector: Selector = Selector::parse("span").unwrap();
    for span in parsed_html.select(&span_selector) {
        if let FIRST_CC_NAME.to_string() = span.inner_html() {
            return span;
        }
    }
    panic!("{} not found in <span> tags.", FIRST_CC_NAME);
}

const LAST_FAN_NAME: String = "".to_string();

// https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/
// https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything
//
/* Document is actually of this form.
 * <p><strong>content creator</strong></p>
 * <p><a href="...">fan name</a></p>
*/

struct AmaExchange {
    cc_name: String, // ElementRef::inner_html
    fan_name: String, // ElementRef::inner_html
    exchange_url: &str, // ElementRef::attr
}

fn compile_ama_index(starting_node: ElementRef) -> Vec<AmaExchange> {
    let mut ama_index: Vec<AmaExchange> = Vec::new();
    let mut cc_name: String = String::new();
    let mut num_loops: u8 = 0;
    let mut current_node: ElementRef = starting_node;
    let max_numloops: u8 = 200;
    loop {
        match *(current_node.value()).name() {
            "span" => {
                cc_name = current_node.inner_html();
            }
            "a" => {
                let fan_name: String = current_node.inner_html();
                let exchange_url: &str = match current_node.attr("href") {
                    Ok(href) => href,
                    Err(href404) => panic!("Error for {fan_name}'s question for {cc_name}: {:?}", href404),
                };
                let ama_exchange: AmaExchange = AmaExchange {
                    cc_name,
                    fan_name,
                    exchange_url,
                };
                ama_index.push(ama_exchange);
                /*
                if let format!("{LAST_FAN_NAME}") = fan_name {
                    break;
                }
                */
            }
            _ => {
                // TODO: Break?
                panic!("Invalid tag.");
            }
        }
        current_node = current_node.next_sibling();
        num_loops += 1;
        if num_loops > max_numloops {
            break;
        }
    }
    ama_index
}
