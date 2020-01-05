extern crate getopts;
extern crate protoc_rust;

use getopts::Options;
use protoc_rust::Customize;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn print_usage(program: &str, opts: Options) {
    let help = format!("Usage:{} [options]", program);
    println!("{}", opts.usage(&help));
}

fn get_files(dir: &PathBuf) -> io::Result<Option<Vec<PathBuf>>> {
    if !dir.is_dir() {
        return Ok(None);
    }

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(sub_files) = get_files(&path)? {
                files.extend(sub_files.iter().cloned());
            }
        } else {
            files.push(path);
        }
    }

    if files.len() != 0 {
        return Ok(Some(files));
    }

    return Ok(None);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("o", "out_dir", "output directory", "[out_dir]");
    opts.optopt(
        "s",
        "src_dir",
        "directory where .proto files reside in",
        "[src_dir]",
    );
    opts.optopt(
        "f",
        "src_files",
        "white space separated .proto files path relative to src_dir, if not set, use all .proto files under [src_dir]",
        "[proto files]",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let out_dir = match matches.opt_str("o") {
        Some(v) => PathBuf::from(&v),
        None => panic!("out_dir can not be empty"),
    };

    let src_dir = match matches.opt_str("s") {
        Some(v) => PathBuf::from(&v),
        None => panic!("src_dir can not be empty"),
    };

    let src_files: Vec<PathBuf> = match matches.opt_str("f") {
        Some(files) => {
            // parse file list separated by white space
            files.split_whitespace().map(|f| PathBuf::from(f)).collect()
        }
        None => {
            let err_msg = &format!("failed to read dir {}", src_dir.to_str().unwrap());
            let all_files = match get_files(&src_dir).expect(err_msg) {
                Some(v) => v,
                None => panic!("no proto files found"),
            };

            all_files
                .into_iter()
                .filter_map(|f| match f.extension() {
                    None => None,
                    Some(ext) => {
                        if ext == "proto" {
                            Some(PathBuf::from(f.strip_prefix(&src_dir).unwrap()))
                        } else {
                            None
                        }
                    }
                })
                .collect()
        }
    };

    for proto in src_files.iter() {
        let sub_dir = match proto.parent() {
            None => PathBuf::new(),
            Some(p) => PathBuf::from(p),
        };

        let dest = out_dir.join(sub_dir);
        if !dest.exists() {
            fs::create_dir_all(&dest)
                .expect(&format!("failed to make dir {}", dest.to_str().unwrap()));
        }

        println!(
            "{} ====> {}",
            src_dir.join(&proto).to_str().unwrap(),
            dest.join(proto.file_name().unwrap()).to_str().unwrap(),
        );

        protoc_rust::run(protoc_rust::Args {
            out_dir: dest.to_str().unwrap(),
            input: &[src_dir.join(proto).to_str().unwrap()],
            includes: &[&src_dir.to_str().unwrap()],
            customize: Customize {
                ..Default::default()
            },
        })
        .expect("failed to generate rs from protos");
    }
}
