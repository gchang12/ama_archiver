// TODO: Either write Makefile, or create CLI for this.
use ama_archiver::{compile_queries, compile_index, write_filetree, fix_database};

// compile index
// correct the database.
// compile queries
// write filetree

fn main() {
    //compile_index();
    fix_database();
    // - UPDATE ama_index SET url_id='evw8g9o' WHERE fan_name='Joe_Zt' AND cc_name='Daron Nefcy';
    // - UPDATE ama_index SET url_id='evwbgza' WHERE fan_name='sloppyjeaux' AND cc_name='Adam McArthur';
    //compile_queries();
    //write_filetree();
}
