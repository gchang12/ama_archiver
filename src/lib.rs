// https://docs.rs/scraper/latest/scraper/
// https://docs.rs/ureq/latest/ureq/

// https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/
// https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything
//
/* Document is actually of this form.
 * <p><strong>content creator</strong></p>
 * <p><a href="...">fan name</a></p>
*/

pub mod ama_indexer {
    use ureq;
    use ego_tree::NodeRef;
    use std::fs;
    use scraper::{Html, Selector, ElementRef};

    const LC_URL: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/";
    pub const LC_FNAME: &str = "link-compendium";
    pub const ODIR_NAME: &str = "output";
    const FIRST_CC_NAME: &str = "Daron Nefcy:";
    const URL_TEMPLATE: &str = "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3";

    #[derive(Debug)]
    pub struct AmaRecord {
        cc_name: String, // ElementRef::inner_html
        fan_name: String, // ElementRef::inner_html
        url: String, // (ElementRef::attr).to_string()
    }

    pub fn fetch_raw_index() -> String {
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

    pub fn save_raw_index(raw_html: String, odir_name: &str, lc_fname: &str) -> () {
        // create 'output' directory
        // save 'raw_html' to {ODIR_NAME}/{LC_FNAME}.html
        let () = match fs::create_dir(odir_name) {
            Ok(()) => println!("'{}' directory successfully created.", odir_name),
            Err(_) => eprintln!("Error creating '{}' directory. Does it exist already?", odir_name),
        };
        let full_opath: String = format!("{}/{}.html", odir_name, lc_fname);
        let () = match fs::write(&full_opath, raw_html) {
            Ok(()) => println!("Contents of (raw_html, String) written to '{}'", full_opath),
            Err(_) => eprintln!("Unable to write (raw_html, String) to '{}'", full_opath),
        };
    }

    pub fn compile_ama_index(raw_html: String, start_text: &str) -> Vec<AmaRecord> {
        // locate the starting node
        let parsed_html: Html = Html::parse_document(&raw_html);
        let strong_selector: Selector = Selector::parse("strong").unwrap();
        let mut node_opt: Option<NodeRef<_>> = None; // used to store the 'current_node'
        //let mut current_node: ElementRef;
        for strong in parsed_html.select(&strong_selector) {
            if strong.inner_html() == start_text.to_string() {
                println!("strong: {:?}", strong.inner_html());
                node_opt = Some(strong.parent().unwrap());
                break;
            }
        }
        let mut current_node = match node_opt {
            Some(node) => node,
            None => panic!("<strong> node that contains '{}' not found. Fatal. Aborting.", start_text),
        };
        // drop(node_opt); // warned against this.
        // begin to compile records
        let mut ama_index: Vec<AmaRecord> = Vec::new();
        let mut cc_name: String = start_text.to_string();
        let mut num_loops: u32 = 0;
        let tolerance: u32 = 100;
        for p in current_node.next_siblings() {
            if let Some(node) = p.first_child() {
                let element_ref: ElementRef = ElementRef::wrap(node).unwrap();
                match element_ref.value().name() {
                    "strong" => {
                        cc_name = element_ref.inner_html();
                        assert_eq!(
                            match cc_name.pop() {
                                Some(colon) => colon,
                                None => panic!("<strong> tag was empty. Inspect."),
                            },
                        ':');
                    },
                    "a" => {
                        let fan_name: String = element_ref.inner_html();
                        let url: String = element_ref.attr("href").unwrap().to_string();
                        let ama_record: AmaRecord = AmaRecord {
                            cc_name: cc_name.clone(),
                            fan_name,
                            // TODO: Create function to find URL template, and isolate the url_id
                            url,
                        };
                        ama_index.push(ama_record);
                    },
                    other => {
                        eprintln!("Unexpected node found. Neither strong nor a: {:?}", other);
                        break;
                    },
                }
            }
        };
        ama_index
    }

    /*
    fn identify_duplicates(ama_index_ref: &Vec<AmaRecord>) -> Vec<AmaRecord> {
        let mut urlid_list: Vec<String> = Vec::new();
        let mut dup_list: Vec<AmaRecord> = Vec::new();
        for ama_record in (*ama_index_ref).iter() {
            if let true = urlid_list.contains(&ama_record.url) {
                dup_list.push(*ama_record);
            }
        }
        for dup in dup_list.iter() {
            eprintln!("{:?}", dup);
        }
        dup_list
    }

    fn save_ama_index(ama_index: Vec<AmaRecord>) {
        // TODO: Learn rusqlite module first.
    }

    fn load_ama_index(full_dbpath: !) {
        // TODO: Learn std::path module first.
    }
    */

    pub fn get_url(url_id: String) -> String {
        // url_template = "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3"
        let mut url_parts: Vec<&str> = URL_TEMPLATE.to_string().split("/").collect();
        url_parts[url_parts.len() - 2] = url_id.as_str();
        // 0: 'https:'
        // 1: ''
        // 2: 'www.reddit.com'
        url_parts[2] = "old.reddit.com";
        let url: String = url_parts.join("/");
        url
    }

    pub fn get_urlid(url: String) -> String {
        // url_template = "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3"
        let url_parts: Vec<&str> = url.split("/").collect();
        let url_id: String = url_parts[url_parts.len() - 2].to_string();
        url_id
    }

}

// TODO: Learn how to write better tests.
// TODO: Learn how to structure the project better.
#[cfg(test)]
mod ama_indexer_tests {
    use crate::ama_indexer;
    use std::fs;

    #[test]
    fn test_get_url() {
        let url_id: String = "nyet".to_string();
        let expected: String = format!("{}/{}/{}", "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything", url_id, "?context=3");
        let actual: String = ama_indexer::get_url(url_id);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_urlid() {
        let expected: String = "nyet".to_string();
        let url: String = format!("{}/{}/{}", "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything", expected, "?context=3");
        let actual: String = ama_indexer::get_urlid(url);
        assert_eq!(actual, expected);
    }

    //#[test]
    fn test_fetch_raw_index() {
        /*
        let raw_index: String = ama_indexer::fetch_raw_index();
        // TODO: Figure out how to show output. `--nocapture` option
        let () = match fs::write("test_fetch_raw_index__results.html", raw_index) {
            Ok(_) => {},
            Err(_) => {},
        };
        //println!("{}", raw_index); // cargo test does not print to stdout
        */
    }

    #[test]
    fn test_compile_ama_index() {
        let full_opath: String = format!("{}/{}.html", ama_indexer::ODIR_NAME, ama_indexer::LC_FNAME);
        let raw_index: String = fs::read_to_string(full_opath).unwrap();
        let start_text: &str = "Daron Nefcy:";
        let ama_index: Vec<ama_indexer::AmaRecord> = ama_indexer::compile_ama_index(raw_index, start_text);
    }
}
