extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;
extern crate image;
extern crate img_hash;

use std::io::{self, BufWriter, Stderr, Stdout, Write};

use docopt::Docopt;
use img_hash::{ImageHash, HashType};
use walkdir::{DirEntry, WalkDir};

const USAGE: &'static str = "
Usage:
    rdedup [options] [<dir> ...]
Options:
    -h, --help
    -L, --follow-links       Follow symlinks.
    --min-depth NUM          Minimum depth.
    --max-depth NUM          Maximum depth.
    -n, --fd-max NUM         Maximum open file descriptors. [default: 32]
    -x, --same-file-system   Stay on the same file system.
";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Args {
    arg_dir: Option<Vec<String>>,
    flag_follow_links: bool,
    flag_min_depth: Option<usize>,
    flag_max_depth: Option<usize>,
    flag_fd_max: usize,
    flag_same_file_system: bool,
}

macro_rules! wout { ($($tt:tt)*) => { {writeln!($($tt)*)}.unwrap() } }

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn print_image_details(out: &mut BufWriter<Stdout>, eout: &mut Stderr, dent: DirEntry) {
    let image = image::open(dent.path());
    match image {
            Err(err) => {
                out.flush().unwrap();
                wout!(eout, "ERROR: {}", err);
            }
            Ok(image) => {
                let hash = ImageHash::hash(&image, 8, HashType::DCT);

                let path = dent.path().canonicalize();
                wout!(out, "{}\t{}", hash.to_base64(), path.unwrap().display());
            }
        }    
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    let mind = args.flag_min_depth.unwrap_or(0);
    let maxd = args.flag_max_depth.unwrap_or(::std::usize::MAX);

    for dir in args.arg_dir.unwrap_or(vec![".".to_string()]) {
        let walkdir = WalkDir::new(dir)
            .max_open(args.flag_fd_max)
            .follow_links(args.flag_follow_links)
            .min_depth(mind)
            .max_depth(maxd)
            .same_file_system(args.flag_same_file_system);

        let it = walkdir.into_iter();
        let mut out = io::BufWriter::new(io::stdout());
        let mut eout = io::stderr();
        for dent in it.filter_entry(|e| !is_hidden(e)) {
            match dent {
                Err(err) => {
                    out.flush().unwrap();
                    wout!(eout, "ERROR: {}", err);
                }
                Ok(dent) => {
                    if !dent.file_type().is_dir() {
                        print_image_details(&mut out, &mut eout, dent);
                    }
                }
            }
        }
    }
}