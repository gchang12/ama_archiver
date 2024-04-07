/*
use std::fs;
use std::path::Path;
*/

use ama_archiver::{compile_queries, compile_index, write_filetree};

/*
use ama_archiver::ama_indexer;
use ama_archiver::ama_scraper;
*/

/*
const LC_FNAME: &str = "link-compendium";
const ODIR_NAME: &str = "output";
const LC_URL: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/clnrdv/link_compendium_of_questions_and_answers_from_the/";
const FIRST_CC_NAME: &str = "Daron Nefcy:";
const DB_FNAME: &str = "ama_archive.db";
*/

fn main() {
    write_filetree();
}

/*
fn compile_queries() -> () {
    // pseudo-constant
    let full_dbpath: &str = "output/ama_archive.db";
    let ama_index: Vec<ama_indexer::AmaRecord> = ama_indexer::load_ama_index(full_dbpath);
    let () = ama_scraper::create_db(full_dbpath);
    let scraped_ama_queries: Vec<ama_scraper::AmaQuery> = ama_scraper::load_ama_queries_from_db(full_dbpath);
    let scraped_urls: Vec<String> = scraped_ama_queries.into_iter().map(|query| query.url_id).collect();
    let record_total = ama_index.len();
    for (recordno, ama_record) in ama_index.into_iter().enumerate() {
        if scraped_urls.contains(&ama_record.url_id) {
            continue;
        }
        println!("Scraping record {}/{} for 'url_id': {}.", recordno + 1, record_total, &ama_record.url_id);
        let mut fetched_ama_query = ama_scraper::AmaQuery {
            url_id: ama_record.url_id.clone(),
            question_text: None,
            answer_text: None,
        };
        let url_id: String = ama_record.url_id;
        let url: String = ama_indexer::get_url(url_id);
        let mut num_attempts: u32 = 1;
        while let None = fetched_ama_query.answer_text {
            println!("Fetching record... Attempt: {}", num_attempts);
            let () = ama_scraper::fetch_ama_query(&url, &mut fetched_ama_query);
            num_attempts += 1;
        };
        let _ = ama_scraper::save_ama_query_to_db(fetched_ama_query, full_dbpath);
    };
    println!("All {} queries have been scraped.", record_total);
}

fn compile_index() -> () {
    // If the file DNE, then scrape the index off the source, and save it to disk.
    let raw_htmlfile: String = format!("{}/{}.html", ODIR_NAME, LC_FNAME);
    let raw_htmlpath: &Path = Path::new(&raw_htmlfile);
    if !raw_htmlpath.exists() {
        let raw_html: String = ama_indexer::fetch_raw_index(LC_URL);
        let () = ama_indexer::save_raw_index(raw_html, ODIR_NAME, LC_FNAME);
    };
    // Grab text off file, and convert it to AmaRecord format.
    let raw_html: String = fs::read_to_string(raw_htmlfile).unwrap();
    let mut ama_index: Vec<ama_indexer::AmaRecord> = ama_indexer::compile_ama_index(raw_html, FIRST_CC_NAME);
    // Do some data finalizing, and then save ama index
    for ama_record in &mut ama_index {
        // 0507
        let url_id: String = ama_record.url_id.clone();
        ama_record.url_id = ama_indexer::get_urlid(url_id);
    };
    let full_dbpath: String = format!("{}/{}", ODIR_NAME, DB_FNAME);
    let () = ama_indexer::create_db(&full_dbpath);
    match ama_indexer::save_ama_index(ama_index, &full_dbpath) {
        Ok(num_bytes) => println!("{} bytes written.", num_bytes),
        Err(save_err) => eprintln!("Could not save to disk: {:?}", save_err),
    };
}
*/
