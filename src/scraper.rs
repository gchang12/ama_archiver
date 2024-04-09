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


/// This module contains functions that fetch and store queries from the source.
/// - fetch_ama_query: Fetches text Q&A data from Reddit as text, and returns it as a dict[str, str].
/// - fetch_ama_queries: Iterates over index, and fetches Q&A data for each entry in the index.
/// - save_ama_query: Saves a given ama_query, provided it's got the right fields.
pub mod ama_scraper {
    use scraper::{Html, Selector};
    use std::path::Path;
    use rusqlite;
    use scraper::ElementRef;

    /// Contains results of fetching from source URLs
    #[derive(PartialEq)]
    #[derive(Debug)]
    pub struct AmaQuery {
        pub url_id: String,
        pub question_text: Option<String>,
        pub answer_text: Option<String>,
    }

    /// Helper function to get desired text off page.
    pub fn get_html_text(usertext_node: ElementRef) -> Option<String> {
        let mut buffer: String = String::new();
        for text in usertext_node.text() {
            buffer.push_str(text);
        }
        Some(buffer)
    }

    /// Fetches `question_text` and `answer_text` values for a given URL.
    /// - url: source whence data is to be fetched.
    /// - ama_query: dict to store fetched data. Initialize outside function.
    pub fn fetch_ama_query(url: &str, ama_query: &mut AmaQuery) -> () {
        let request: ureq::Request = ureq::get(url);
        let raw_html: String = match request.call() {
            Ok(resp) => resp.into_string().unwrap(),
            Err(reqerr) => panic!("Unable to get response from '{}': {:?}", url, reqerr),
        };
        let parsed_html: Html = Html::parse_document(&raw_html);
        let usertextbody_selector: Selector = Selector::parse(".usertext-body").unwrap();
        for (commentno, usertext_node) in parsed_html.select(&usertextbody_selector).enumerate() {
            match commentno {
                0 => continue,
                1 => ama_query.question_text = get_html_text(usertext_node),
                2 => ama_query.answer_text = get_html_text(usertext_node),
                _ => eprintln!("Extraneous node found for url_id: {:?}.", &ama_query.url_id),
            }
        }
    }

    /// Creates database file with specified filename, and initializes queries table.
    pub fn create_db(full_dbpath: &str) -> () {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        match cnxn.execute(
            "CREATE TABLE ama_queries (
                url_id TEXT PRIMARY KEY,
                question_text TEXT NOT NULL,
                answer_text TEXT NOT NULL
            );",
            ()
        ) {
            Ok(_) => println!("ama_queries table created in '{}'.", full_dbpath),
            Err(_) => panic!("The table 'ama_queries' already exists in '{}'. Aborting.", full_dbpath),
        };
    }

    /// Creates 'ama_queries' table in `full_dbpath`, and saves `ama_query` into the table.
    /// - ama_query: populated dict to be loaded into the database.
    /// - full_dbpath: tells the function where the database file is.
    pub fn save_ama_query_to_db(ama_query: AmaQuery, full_dbpath: impl AsRef<Path>) -> rusqlite::Result<usize> {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        // Begin data dump here.
        cnxn.execute(
            "INSERT INTO ama_queries VALUES (?1, ?2, ?3);",
            (
                ama_query.url_id,
                ama_query.question_text.unwrap(),
                ama_query.answer_text.unwrap(),
            )
        )?;
        // Learn how to get length of INSERT result.
        Ok(0)
    }

    /// Loads 'ama_queries' table from `full_dbpath` into List[dict].
    /// - full_dbpath: Tells function where to find `ama_queries`
    pub fn load_ama_queries_from_db(full_dbpath: impl AsRef<Path>) -> Vec<AmaQuery> {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        let mut stmt: rusqlite::Statement = cnxn.prepare(
            "SELECT url_id, question_text, answer_text FROM ama_queries;"
            ).unwrap();
        let ama_query_iter = stmt.query_map(
            [],
            |row| {
                Ok(
                    AmaQuery {
                        url_id: row.get(0).unwrap(),
                        question_text: Some(row.get(1).unwrap()),
                        answer_text: Some(row.get(2).unwrap()),
                    }
                )
            }
        ).unwrap();
        let mut ama_queries: Vec<AmaQuery> = Vec::new();
        for ama_query in ama_query_iter {
            ama_queries.push(ama_query.unwrap());
        }
        ama_queries
    }

}

#[cfg(test)]
mod ama_scraper_tests {
    use crate::ama_scraper;
    use super::remove_file;
    use rusqlite;
    use scraper::{Html, Selector};

    #[test]
    fn test_get_html_text() {
        let sample_html: &str = r#"
            <div class="usertext-body" ><div class="md"><p></p></div></div>
            <div class="usertext-body" ><div class="md"><p></p></div></div>
            <div class="usertext-body" ><div class="md"><p>Some of these days, I <em>will</em> survive.</p></div></div>
        "#;
        let expected: Option<String> = Some("Some of these days, I will survive.".to_string());
        let parsed_html: Html = Html::parse_document(sample_html);
        let usertextbody_selector: Selector = Selector::parse(".usertext-body").unwrap();
        let mut actual: Option<String> = None;
        for usertext_node in parsed_html.select(&usertextbody_selector) {
            actual = ama_scraper::get_html_text(usertext_node);
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fetch_ama_query() {
        let url: &str = "https://old.reddit.com/r/StarVStheForcesofEvil/comments/cll9u5/star_vs_the_forces_of_evil_ask_me_anything/evw3fne/?context=3";
        let mut ama_query = ama_scraper::AmaQuery {
            url_id: "evw3fne".to_string(),
            question_text: None,
            answer_text: None,
        };
        while let None = ama_query.answer_text {
            let () = ama_scraper::fetch_ama_query(url, &mut ama_query);
        }
        if let None = ama_query.question_text {
            panic!("ama_query.question_text is unexpectedly None. Inspect!");
        }
        if let None = ama_query.answer_text {
            panic!("ama_query.answer_text is unexpectedly None. Inspect!");
        }
    }

    #[test]
    fn test_save_ama_query_to_db() {
        let ama_query = ama_scraper::AmaQuery {
            url_id: "url_id".to_string(),
            question_text: Some("question_text".to_string()),
            answer_text: Some("answer_text".to_string()),
        };
        let outdir: &str = "output";
        let filename: &str = "ama_query-save_test.db";
        let full_dbpath: String = format!("{}/{}", outdir, filename);
        // remove_file(&full_dbpath);
        let () = ama_scraper::create_db(&full_dbpath);
        match ama_scraper::save_ama_query_to_db(ama_query, &full_dbpath) {
            Ok(_) => println!("AmaQuery successfully saved to database."),
            Err(sql_save_err) => panic!("Problem saving to database: {:?}", sql_save_err),
        };
        // Load from the database, and verify record.
        let cnxn: rusqlite::Connection = get_db_cnxn(&full_dbpath);
        let mut stmt: rusqlite::Statement = cnxn.prepare(
            "SELECT url_id, question_text, answer_text FROM ama_queries;"
            ).unwrap();
        let ama_query_iter = stmt.query_map(
            [],
            |row| {
                Ok(
                    ama_scraper::AmaQuery {
                        url_id: row.get(0).unwrap(),
                        question_text: Some(row.get(1).unwrap()),
                        answer_text: Some(row.get(2).unwrap()),
                    }
                )
            }
        ).unwrap();
        let expected = ama_scraper::AmaQuery {
            url_id: "url_id".to_string(),
            question_text: Some("question_text".to_string()),
            answer_text: Some("answer_text".to_string()),
        };
        let mut actual = ama_scraper::AmaQuery {
            url_id: String::new(),
            question_text: None,
            answer_text: None,
        };
        for ama_query in ama_query_iter {
            actual = ama_query.unwrap();
        }
        assert_eq!(actual, expected);
        remove_file(full_dbpath);
    }

    fn get_db_cnxn(full_dbpath: &str) -> rusqlite::Connection {
        let cnxn: rusqlite::Connection = rusqlite::Connection::open(full_dbpath).unwrap();
        match cnxn.execute(
            "CREATE TABLE IF NOT EXISTS ama_queries (
                url_id TEXT PRIMARY KEY,
                question_text TEXT NOT NULL,
                answer_text TEXT NOT NULL
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
    fn test_load_ama_queries_from_db() {
        let full_dbpath: &str = "output/ama_query-load_test.db";
        let cnxn: rusqlite::Connection = get_db_cnxn(full_dbpath);
        // Begin data dump here.
        let ama_query1 = ama_scraper::AmaQuery {
            url_id: "url_id".to_string(),
            question_text: Some("question_text".to_string()),
            answer_text: Some("answer_text".to_string()),
        };
        let ama_query2 = ama_scraper::AmaQuery {
            url_id: "url_id2".to_string(),
            question_text: Some("question_text2".to_string()),
            answer_text: Some("answer_text2".to_string()),
        };
        let expected: Vec<ama_scraper::AmaQuery> = Vec::from(
            [
                ama_query1,
                ama_query2,
            ]
        );
        for ama_query in &expected {
            let _ = cnxn.execute(
                "INSERT INTO ama_queries VALUES (?1, ?2, ?3);",
                (
                    // E0507
                    (*ama_query).url_id.clone(),
                    (*ama_query).question_text.clone().unwrap(),
                    (*ama_query).answer_text.clone().unwrap(),
                )
            ).unwrap();
        }
        let actual = ama_scraper::load_ama_queries_from_db(full_dbpath);
        assert_eq!(actual, expected);
        remove_file(full_dbpath);
    }

}
