// https://docs.rs/scraper/latest/scraper/
// https://docs.rs/ureq/latest/ureq/

// https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/
// https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything
//
/* Document is actually of this form.
 * <p><strong>content creator</strong></p>
 * <p><a href="...">fan name</a></p>
*/
use ureq;
use std::fs;
use scraper::{Html, Selector, ElementRef};

const LC_URL: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/";
const LC_FNAME: &str = "link-compendium";
const ODIR_NAME: &str = "output";
const FIRST_CC_NAME: &str = "Daron Nefcy";

struct AmaRecord {
    cc_name: String, // ElementRef::inner_html
    fan_name: String, // ElementRef::inner_html
    url: String, // (ElementRef::attr).to_string()
}

fn fetch_raw_index() -> String {
    // use ureq to get text of LC_URL
    // save text into html file in output
    // ensure output/ exists beforehand
    let request: ureq::Request = ureq::get(LC_URL);
    let raw_html: String = match request.call() {
        Ok(resp) => resp.into_string().unwrap(),
        Err(reqerr) => panic!("Unable to get response from '{}': {:?}", LC_URL, reqerr),
    };
    raw_html
}

fn save_raw_index(raw_html: String) -> () {
    // create 'output' directory
    // save 'raw_html' to {ODIR_NAME}/{LC_FNAME}.html
    let () = match fs::create_dir(ODIR_NAME) {
        Ok(()) => println!("'{}' directory successfully created.", ODIR_NAME),
        Err(_) => eprintln!("Error creating '{}' directory. Does it exist already?", ODIR_NAME),
    };
    let full_opath: String = format!("{}/{}.html", ODIR_NAME, LC_FNAME);
    let () = match fs::write(&full_opath, raw_html) {
        Ok(()) => println!("Contents of (raw_html, String) written to '{}'", full_opath),
        Err(_) => eprintln!("Unable to write (raw_html, String) to '{}'", full_opath),
    };
}

fn compile_ama_index(raw_html: String) -> Vec<AmaRecord> {
    // locate the starting node
    let parsed_html: Html = Html::parse_document(&raw_html);
    let strong_selector: Selector = Selector::parse("strong").unwrap();
    let mut node_vec: Vec<ElementRef> = Vec::new(); // used to store the 'current_node'
    //let mut current_node: ElementRef;
    for strong in parsed_html.select(&strong_selector) {
        if strong.inner_html() == FIRST_CC_NAME.to_string() {
            //current_node = strong;
            node_vec.push(strong);
            break;
        }
    }
    let mut current_node = match node_vec.pop() {
        Some(node) => node,
        None => panic!("<strong> node that contains '{}' not found. Fatal. Aborting.", FIRST_CC_NAME),
    };
    // begin to compile records
    let mut ama_index: Vec<AmaRecord> = Vec::new();
    let mut cc_name: String = String::new();
    loop {
        match current_node.value().name() {
            "strong" => {
                cc_name = current_node.inner_html();
            },
            "a" => {
                let fan_name: String = current_node.inner_html();
                let url: String = current_node.attr("href").unwrap().to_string();
                let ama_record: AmaRecord = AmaRecord {
                    cc_name: cc_name.clone(),
                    fan_name,
                    url,
                };
                ama_index.push(ama_record);
            },
            _ => {
                break;
            },
        }
        current_node = ElementRef::wrap(
            current_node.parent().unwrap().next_sibling().unwrap()
            ).unwrap();
    }
    ama_index
}

fn main() {
    // TODO: Learn project organization
    // TODO: Write tests
    // TODO: Reanalyze code to see if the logic works wrt actual tree
}
