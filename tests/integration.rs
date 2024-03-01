#[cfg(test)]
mod integration {
    use ama_archiver::ama_indexer;
    #[test]
    fn test_fetch_and_save_index() {
        let raw_html: String = ama_indexer::fetch_raw_index();
        let () = ama_indexer::save_raw_index(raw_html);
    }
}
