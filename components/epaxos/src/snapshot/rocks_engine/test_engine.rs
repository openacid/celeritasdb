use tempfile::Builder;

use super::super::RocksDBEngine;
use crate::snapshot::test_engine::*;

#[test]
fn test_base() {
    let mut eng = new_rocks_engine();
    test_base_trait(&mut eng);
}

fn new_rocks_engine() -> RocksDBEngine {
    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    return RocksDBEngine::new(&db_path).unwrap();
}
