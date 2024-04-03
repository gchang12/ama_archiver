use std::path::Path;
use std::fs;

fn remove_file(full_path: impl AsRef<Path> + std::fmt::Debug) -> () {
    match fs::remove_file(&full_path) {
        Ok(()) => {
            println!("File removed: {:?}", &full_path);
        },
        Err(rm_err) => {
            eprintln!("File not removed: {:?}", rm_err);
        },
    };
}

pub mod ama_indexer {
    use ureq;
    use ego_tree::NodeRef;
    use std::fs;
    use scraper::{Html, Selector, ElementRef};
    use rusqlite;
    use std::path::Path;

    const DB_FNAME: &str = "ama_archive.db";
    const FIRST_CC_NAME: &str = "Daron Nefcy:";
    const LC_URL: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/";
    const LC_FNAME: &str = "link-compendium";
    pub const ODIR_NAME: &str = "output";
    // '/'-split list must be modified:
    // -2: '' -> {url_id}
    const URL_TEMPLATE: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3";

    #[derive(PartialEq)]
    #[derive(Debug)]
    pub struct AmaRecord {
        pub cc_name: String, // ElementRef::inner_html
        pub fan_name: String, // ElementRef::inner_html
        pub url_id: String, // (ElementRef::attr).to_string()
    }

    pub fn fetch_raw_index(url: &str) -> String {
        // use ureq to get text of LC_URL
        // save text into html file in output
        // ensure output/ exists beforehand
        let request: ureq::Request = ureq::get(url);
        let raw_html: String = match request.call() {
            Ok(resp) => resp.into_string().unwrap(),
            Err(reqerr) => panic!("Unable to get response from '{}': {:?}", url, reqerr),
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
        let current_node = match node_opt {
            Some(node) => node,
            None => panic!("<strong> node that contains '{}' not found. Fatal. Aborting.", start_text),
        };
        // drop(node_opt); // warned against this.
        // begin to compile records
        let mut ama_index: Vec<AmaRecord> = Vec::new();
        let mut cc_name: String = start_text.to_string();
        assert_eq!(
            match cc_name.pop() {
                Some(colon) => colon,
                None => panic!("<strong> tag was empty. Inspect."),
            },
        ':');
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
                        let url_id: String = element_ref.attr("href").unwrap().to_string();
                        let ama_record: AmaRecord = AmaRecord {
                            cc_name: cc_name.clone(),
                            fan_name,
                            url_id,
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
    fn identify_duplicates(ama_index_ref: &Vec<AmaRecord>) -> () {
        // Identify duplicate url_id rows:
        // - SELECT * FROM ama_index WHERE url_id IN (SELECT url_id FROM ama_index GROUP BY url_id HAVING COUNT(url_id) > 1);
        // Make corrections:
        // - UPDATE ama_index SET url_id='evw8g9o' WHERE fan_name='Joe_Zt' AND cc_name='Daron Nefcy';
        // - UPDATE ama_index SET url_id='evwbgza' WHERE fan_name='sloppyjeaux' AND cc_name='Adam McArthur';
        let mut urlid_list: Vec<String> = Vec::new();
        let mut dup_list: Vec<AmaRecord> = Vec::new();
        for ama_record_ref in (*ama_index_ref).iter() {
            if let true = urlid_list.contains(&((*ama_record_ref).url_id)) {
                eprintln!("{:?}", *ama_record_refj);
            }
        };
    }
    */

    pub fn create_db(full_dbpath: &str) -> () {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        match cnxn.execute(
            "CREATE TABLE ama_index (
                url_id TEXT,
                cc_name TEXT,
                fan_name TEXT
            );",
            ()
        ) {
            Ok(_) => println!("'ama_index' table has been created in '{}'.", full_dbpath),
            Err(_) => panic!("Table 'ama_index' already exists in '{}'. Aborting.", full_dbpath),
        };
    }

    pub fn save_ama_index(ama_index: Vec<AmaRecord>, full_dbpath: &str) -> rusqlite::Result<usize> {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        let ama_index_len: usize = ama_index.len();
        // Begin data dump here.
        for ama_record in ama_index {
            cnxn.execute(
                "INSERT INTO ama_index VALUES (?1, ?2, ?3);",
                (
                    ama_record.url_id,
                    ama_record.cc_name,
                    ama_record.fan_name,
                )
            )?;
        };
        Ok(ama_index_len)
    }

    pub fn load_ama_index(full_dbpath: &str) -> Vec<AmaRecord> {
        let mut ama_index: Vec<AmaRecord> = Vec::new();
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        let mut stmt: rusqlite::Statement = cnxn.prepare(
            "SELECT url_id, cc_name, fan_name FROM ama_index;"
            ).unwrap();
        let ama_record_iter = stmt.query_map(
            [],
            |row| {
                Ok(
                    AmaRecord {
                        url_id: row.get(0).unwrap(),
                        cc_name: row.get(1).unwrap(),
                        fan_name: row.get(2).unwrap(),
                    }
                )
            }
        ).unwrap();
        for ama_record in ama_record_iter {
            ama_index.push(ama_record.unwrap());
        };
        ama_index
    }

    pub fn get_url(url_id: String) -> String {
        // url_template = "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3"
        let url_template: String = URL_TEMPLATE.to_string();
        let mut url_parts: Vec<&str> = url_template.split("/").collect();
        let urlid_loc: usize = url_parts.len() - 2;
        url_parts[urlid_loc] = url_id.as_str();
        // 0: 'https:'
        // 1: ''
        // 2: 'www.reddit.com'
        //url_parts[2] = "old.reddit.com";
        let url: String = url_parts.join("/");
        url
    }

    pub fn get_urlid(url: String) -> String {
        // url_template = "https://www.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything//?context=3"
        let url_parts: Vec<&str> = url.split("/").collect();
        let urlid_loc: usize = url_parts.len() - 2;
        let url_id: String = url_parts[urlid_loc].to_string();
        url_id
    }

    pub fn main() -> () {
        // If the file DNE, then scrape the index off the source, and save it to disk.
        let raw_htmlfile: String = format!("{}/{}.html", ODIR_NAME, LC_FNAME);
        let raw_htmlpath: &Path = Path::new(&raw_htmlfile);
        if !raw_htmlpath.exists() {
            let raw_html: String = fetch_raw_index(LC_URL);
            let () = save_raw_index(raw_html, ODIR_NAME, LC_FNAME);
        };
        // Grab text off file, and convert it to AmaRecord format.
        let raw_html: String = fs::read_to_string(raw_htmlfile).unwrap();
        let mut ama_index: Vec<AmaRecord> = compile_ama_index(raw_html, FIRST_CC_NAME);
        // Do some data finalizing, and then save ama index
        for ama_record in &mut ama_index {
            // 0507
            let url_id: String = ama_record.url_id.clone();
            ama_record.url_id = get_urlid(url_id);
        };
        let full_dbpath: String = format!("{}/{}", ODIR_NAME, DB_FNAME);
        let () = create_db(&full_dbpath);
        match save_ama_index(ama_index, &full_dbpath) {
            Ok(num_bytes) => println!("{} bytes written.", num_bytes),
            Err(save_err) => eprintln!("Could not save to disk: {:?}", save_err),
        };
    }

}


#[cfg(test)]
mod ama_indexer_tests {
    use crate::ama_indexer;
    use std::fs;
    use super::remove_file;

    fn get_raw_index() -> String {
        let raw_index: &str = r#"
            <p><strong>cc_name1:</strong></p>

            <p><a href="1">fan_name1</a></p>
            <p><a href="2">fan_name2</a></p>
            <p><a href="1">fan_name3</a></p>
            <hr />
            <p><strong>cc_name2:</strong></p>
            <p><a href="3">fan_name4</a></p>
            <p><a href="4">fan_name5</a></p>
        "#;
        raw_index.to_string()
    }

    #[test]
    fn test_get_url() {
        let url_id: String = "nyet".to_string();
        let expected: String = format!("{}/{}/{}", "https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything", url_id, "?context=3");
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

    #[test]
    fn test_fetch_raw_index() {
        let url: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/";
        let raw_index: String = ama_indexer::fetch_raw_index(url);
        // Few tests to check that it contains some keywords.
        let keywords: Vec<&str> = Vec::from(
            [
                "</html>",
                "Daron Nefcy",
                "Dominic",
                "Hammersley",
                "VeronicaMewniFan",
            ]
        );
        for kw in keywords {
            assert!(raw_index.contains(kw));
        }
    }

    fn get_ama_index() -> Vec<ama_indexer::AmaRecord> {
        let index_tup: Vec<(&str, &str, &str)> = Vec::from(
            [
                ("cc_name1", "fan_name1", "1"),
                ("cc_name1", "fan_name2", "2"),
                ("cc_name1", "fan_name3", "1"),
                ("cc_name2", "fan_name4", "3"),
                ("cc_name2", "fan_name5", "4"),
            ]
        );
        let mut expected: Vec<ama_indexer::AmaRecord> = Vec::new();
        for tup in index_tup.into_iter().map(|field_tup| {
            let (cc_name, fan_name, url_id): (&str, &str, &str) = field_tup;
            (cc_name.to_string(), fan_name.to_string(), url_id.to_string())
        }
        ) {
            let (cc_name, fan_name, url_id): (String, String, String) = tup;
            let ama_record = ama_indexer::AmaRecord {
                cc_name,
                fan_name,
                url_id,
            };
            expected.push(ama_record);
        };
        expected
    }

    #[test]
    fn test_compile_ama_index() {
        // Decided not to rely on fetched data for test.
        // let full_opath: String = format!("{}/{}.html", ama_indexer::ODIR_NAME, ama_indexer::LC_FNAME);
        /*fs::read_to_string(full_opath).unwrap();*/
        let expected: Vec<ama_indexer::AmaRecord> = get_ama_index();
        let start_text: &str = "cc_name1:";
        let raw_index: &str = &get_raw_index();
        let actual: Vec<ama_indexer::AmaRecord> = ama_indexer::compile_ama_index(raw_index.to_string(), start_text);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_save_raw_index() {
        let raw_index: &str = &get_raw_index();
        let odir_name: &str = ama_indexer::ODIR_NAME;
        let lc_fname: &str = "test_save_raw_index-output";
        // Assert that saved text is the same as the loaded text.
        let () = ama_indexer::save_raw_index(raw_index.to_string(), odir_name, lc_fname);
        let full_htmlpath: String = format!("{}/{}.html", odir_name, lc_fname);
        let actual: String = fs::read_to_string(&full_htmlpath).unwrap();
        let expected: String = raw_index.to_string();
        assert_eq!(actual, expected);
        // Cleanup
        remove_file(full_htmlpath);
    }

    #[test]
    fn test_save_ama_index() {
        let ama_index: Vec<ama_indexer::AmaRecord> = get_ama_index();
        let full_dbpath: &str = "output/ama_index-save_test.db";
        // if full_dbpath.exists(): rm full_dbpath
        let () = ama_indexer::create_db(full_dbpath);
        let save_result: Result<usize, _> = ama_indexer::save_ama_index(ama_index, full_dbpath);
        match save_result {
            Ok(numrows) => {
                println!("{} rows written", numrows);
            },
            Err(sql_err) => {
                panic!("SQL error occurred: {:?}",  sql_err);
            },
        };
        // fetch saved database if successful. Lifted straight off front page.
        let cnxn: rusqlite::Connection = get_db_cnxn(full_dbpath);
        let mut stmt: rusqlite::Statement = cnxn.prepare(
            "SELECT url_id, cc_name, fan_name FROM ama_index;"
            ).unwrap();
        let ama_record_iter = stmt.query_map(
            [],
            |row| {
                Ok(
                    ama_indexer::AmaRecord {
                        url_id: row.get(0).unwrap(),
                        cc_name: row.get(1).unwrap(),
                        fan_name: row.get(2).unwrap(),
                    }
                )
            }
        ).unwrap();
        remove_file(full_dbpath);
        let mut actual: Vec<ama_indexer::AmaRecord> = Vec::new();
        for ama_record in ama_record_iter {
            actual.push(ama_record.unwrap());
        };
        let expected: Vec<ama_indexer::AmaRecord> = get_ama_index();
        assert_eq!(actual, expected);
    }

    fn get_db_cnxn(full_dbpath: &str) -> rusqlite::Connection {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        match cnxn.execute(
            "CREATE TABLE IF NOT EXISTS ama_index (
                url_id TEXT,
                cc_name TEXT,
                fan_name TEXT
            );",
            ()
        ) {
            Ok(_) => {
                println!("Table created.");
            },
            Err(sql_err) => {
                eprintln!("SQL error: {:?}", sql_err);
            },
        };
        cnxn
    }

    #[test]
    fn test_load_ama_index() {
        let full_dbpath: &str = "output/ama_index-load_test.db";
        let ama_index: Vec<ama_indexer::AmaRecord> = get_ama_index();
        // Insert into table, then test load.
        let cnxn: rusqlite::Connection = get_db_cnxn(full_dbpath);
        // Begin data dump here.
        for ama_record in ama_index {
            cnxn.execute(
                "INSERT INTO ama_index VALUES (?1, ?2, ?3);",
                (
                    ama_record.url_id,
                    ama_record.cc_name,
                    ama_record.fan_name,
                )
            ).unwrap();
        };
        let actual: Vec<ama_indexer::AmaRecord> = ama_indexer::load_ama_index(full_dbpath);
        let expected: Vec<ama_indexer::AmaRecord> = get_ama_index();
        assert_eq!(actual, expected);
        remove_file(full_dbpath);
    }

}

