use ama_archiver::ama_indexer;
use std::fs;

fn main() {
    let full_opath: String = format!("{}/{}.html", ama_indexer::ODIR_NAME, ama_indexer::LC_FNAME);
    let raw_index: String = fs::read_to_string(full_opath).unwrap();
    let ama_index: Vec<ama_indexer::AmaRecord> = ama_indexer::compile_ama_index(raw_index);
    println!("{:?}", ama_index);
}
