use std::fs;
use std::path::Path;

use crate::DBColumnFamily;
use rocksdb::{ColumnFamilyOptions, DBOptions, DB};

struct CFOptions {
    cf: DBColumnFamily,
    options: ColumnFamilyOptions,
}

impl CFOptions {
    fn new(cf: DBColumnFamily, options: ColumnFamilyOptions) -> CFOptions {
        CFOptions { cf, options }
    }
}

fn get_all_cfs_opts() -> Vec<CFOptions> {
    let mut cfs_opts = Vec::with_capacity(DBColumnFamily::all().len());

    for cf in DBColumnFamily::all() {
        cfs_opts.push(CFOptions::new(cf, ColumnFamilyOptions::new()));
    }

    return cfs_opts;
}

fn db_exists(path: &str) -> Result<bool, String> {
    let db_path = Path::new(path);
    if !db_path.exists() || !db_path.is_dir() {
        return Ok(false);
    }

    match fs::read_dir(path) {
        Ok(mut dir) => {
            return Ok(dir.next().is_some());
        }
        Err(err) => return Err(format!("read path {} failed, got error: {}", path, err)),
    }
}

pub fn open(path: &str) -> Result<DB, String> {
    let mut db_opt = DBOptions::new();

    let cfs_opts = get_all_cfs_opts();

    let mut exist_cfs_opts = vec![];
    let mut new_cfs_opts = vec![];

    if !db_exists(path)? {
        db_opt.create_if_missing(true);

        for x in cfs_opts {
            if x.cf == DBColumnFamily::Default {
                exist_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
            } else {
                new_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
            }
        }

        return open_db_cfs(path, db_opt, new_cfs_opts, exist_cfs_opts);
    }

    db_opt.create_if_missing(false);

    let cf_list = DB::list_column_families(&db_opt, path)?;
    let existed: Vec<&str> = cf_list.iter().map(|v| v.as_str()).collect();
    let needed: Vec<&str> = cfs_opts.iter().map(|x| x.cf.into()).collect();

    if existed == needed {
        return open_db_cfs(path, db_opt, vec![], cfs_opts);
    }

    for x in cfs_opts {
        if existed.contains(&x.cf.into()) {
            exist_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
        } else {
            new_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
        }
    }

    return open_db_cfs(path, db_opt, new_cfs_opts, exist_cfs_opts);
}

fn open_db_cfs(
    path: &str,
    db_opt: DBOptions,
    new_cfs_opts: Vec<CFOptions>,
    exist_cfs_opts: Vec<CFOptions>,
) -> Result<DB, String> {
    let len_exist_cf = exist_cfs_opts.len();
    let len_new_cf = new_cfs_opts.len();

    if len_exist_cf + len_new_cf == 0 {
        return Err(format!("no column family specified"));
    }

    let mut exist_cfs_v: Vec<&str> = Vec::with_capacity(len_exist_cf);
    let mut exist_opts_v = Vec::with_capacity(len_exist_cf);

    for x in exist_cfs_opts {
        exist_cfs_v.push(x.cf.into());
        exist_opts_v.push(x.options);
    }

    let mut db = DB::open_cf(
        db_opt,
        path,
        exist_cfs_v.into_iter().zip(exist_opts_v).collect(),
    )?;

    for x in new_cfs_opts {
        db.create_cf((x.cf.into(), x.options))?;
    }

    return Ok(db);
}

#[test]
fn test_open() {
    use tempfile::Builder;

    let tmp_root = Builder::new().tempdir().unwrap();
    let db_path = format!("{}/test", tmp_root.path().display());
    let db = open(&db_path).unwrap();

    assert_eq!(db.path(), db_path);

    let mut cfs = db.cf_names();
    let mut exp: Vec<&str> = vec![];
    for cf in DBColumnFamily::all() {
        exp.push(cf.into());
    }

    assert_eq!(cfs.sort(), exp.sort());
}
