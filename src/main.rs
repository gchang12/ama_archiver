use ama_archiver::ama_indexer;
use std::path::Path;
use std::fs;

const DB_FNAME: &str = "ama_archive.db";
const LC_FNAME: &str = "link-compendium";
const FIRST_CC_NAME: &str = "Daron Nefcy:";

fn main() {
    compile_index();
}

fn compile_index() -> () {
    // If the file DNE, then scrape the index off the source, and save it to disk.
    let raw_htmlfile: String = format!("{}/{}.html", ama_indexer::ODIR_NAME, LC_FNAME);
    let raw_htmlpath: &Path = Path::new(&raw_htmlfile);
    if !raw_htmlpath.exists() {
        let raw_html: String = ama_indexer::fetch_raw_index(ama_indexer::LC_URL);
        let () = ama_indexer::save_raw_index(raw_html, ama_indexer::ODIR_NAME, LC_FNAME);
    };
    // Grab text off file, and convert it to AmaRecord format.
    let raw_html: String = fs::read_to_string(raw_htmlfile).unwrap();
    let mut ama_index: Vec<ama_indexer::AmaRecord> = ama_indexer::compile_ama_index(raw_html, FIRST_CC_NAME);
    // Do some data finalizing, and then save ama index
    for ama_record in &mut ama_index {
        let url_id: String = ama_record.url_id.clone();
        ama_record.url_id = ama_indexer::get_urlid(url_id);
    };
    let full_dbpath: String = format!("{}/{}", ama_indexer::ODIR_NAME, DB_FNAME);
    match ama_indexer::save_ama_index(ama_index, &full_dbpath) {
        Ok(num_bytes) => println!("{} bytes written.", num_bytes),
        Err(save_err) => eprintln!("Could not save to disk: {:?}", save_err),
    };
}
