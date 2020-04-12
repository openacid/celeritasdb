use std::sync::Arc;
use tempfile::Builder;

use super::super::RocksDBEngine;
use crate::snapshot::test_engine::*;

#[test]
fn test_base() {
    test_base_trait(new_rocks_engine());
}

fn new_rocks_engine() -> Arc<RocksDBEngine> {
    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());

    Arc::new(RocksDBEngine::new(&db_path).unwrap())
}
