use std::fs;
use std::path::Path;

use rocksdb::{ColumnFamilyOptions, DB, DBOptions};

use super::{DBCF, DBPath};

struct CFOptions<'a> {
    cf: &'a str,
    options: ColumnFamilyOptions,
}

impl<'a> CFOptions<'a> {
    pub fn new(cf: &'a str, options: ColumnFamilyOptions) -> CFOptions<'a> {
        CFOptions {cf, options}
    }
}


fn get_all_cfs_opts<'a> () -> Vec<CFOptions<'a>> {

    let mut cfs_opts = Vec::with_capacity(DBCF::all().len());

    for cf in DBCF::all(){
        cfs_opts.push(CFOptions::new(cf, ColumnFamilyOptions::new()));
    }

    return cfs_opts;
}

fn db_exists(path: &str) -> bool {
    let db_path = Path::new(path);
    if !db_path.exists() || !db_path.is_dir() {
        return false;
    }

    return fs::read_dir(&path).unwrap().next().is_some();
}


pub fn open(path: DBPath) -> Result<DB, String> {
    let mut db_opt = DBOptions::new();

    let cfs_opts = get_all_cfs_opts();

    let mut exist_cfs_opts = vec![];
    let mut new_cfs_opts = vec![];

    if !db_exists(path.as_str()) {
        db_opt.create_if_missing(true);

        for x in cfs_opts {
            if x.cf == DBCF::Default.as_str() {
                exist_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
            } else {
                new_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
            }
        }

        return open_db_cfs(path, db_opt, new_cfs_opts, exist_cfs_opts);
    }

    db_opt.create_if_missing(false);

    let cf_list = DB::list_column_families(&db_opt, path.as_str())?;
    let existed: Vec<&str> = cf_list.iter().map(|v| v.as_str()).collect();
    let needed: Vec<&str> = cfs_opts.iter().map(|x| x.cf).collect();

    if existed == needed {
        return open_db_cfs(path, db_opt, vec![], cfs_opts);
    }

    for x in cfs_opts {
        if existed.contains(&x.cf) {
            exist_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
        } else {
            new_cfs_opts.push(CFOptions::new(x.cf, x.options.clone()));
        }
    }

    return open_db_cfs(path, db_opt, new_cfs_opts, exist_cfs_opts);
}

fn open_db_cfs(path: DBPath, db_opt: DBOptions, new_cfs_opts: Vec<CFOptions<'_>>, exist_cfs_opts: Vec<CFOptions<'_>>) -> Result<DB, String>{
    let len_exist_cf = exist_cfs_opts.len();
    let len_new_cf = new_cfs_opts.len();

    if len_exist_cf + len_new_cf == 0{
        return Err(format!("no column family specified"));
    }

    let mut exist_cfs_v = Vec::with_capacity(len_exist_cf);
    let mut exist_opts_v = Vec::with_capacity(len_exist_cf);

    for x in exist_cfs_opts {
        exist_cfs_v.push(x.cf);
        exist_opts_v.push(x.options);
    }

    let mut db = DB::open_cf(db_opt, path.as_str(), exist_cfs_v.into_iter().zip(exist_opts_v).collect())?;

    for x in new_cfs_opts {
        db.create_cf((x.cf, x.options))?;
    }

    return Ok(db);
}
