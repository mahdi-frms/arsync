use arsync::{sync_dirs, SyncMode};
use clap::Parser;
use std::{path::PathBuf, process::exit};

#[derive(Parser, Debug)]
#[clap(version = "0.1.0", about = "file synchronization utility")]
struct Args {
    #[clap(help = "source directory")]
    src: Option<PathBuf>,

    #[clap(help = "destination directory")]
    dest: Option<PathBuf>,

    #[clap(short, long)]
    update: bool,

    #[clap(short, long)]
    soft: bool,

    #[clap(short, long)]
    hard: bool,

    #[clap(short, long)]
    mixed: bool,

    #[clap(short, long)]
    verbose: bool,
}

fn err(str: &str) -> ! {
    println!("{}", str);
    exit(1)
}

const ERR_SRC: &str = "Error: invalid source directory";
const ERR_DEST: &str = "Error: invalid destination directory";

fn main() {
    let args = Args::parse();
    let src = args
        .src
        .unwrap_or_else(|| err("Error: source directory not provided"))
        .canonicalize()
        .unwrap_or_else(|_| err(ERR_SRC));
    let dest = args
        .dest
        .unwrap_or_else(|| err("Error: destination directory not provided"))
        .canonicalize()
        .unwrap_or_else(|_| err(ERR_SRC));

    if std::fs::metadata(&src)
        .unwrap_or_else(|_| err(ERR_SRC))
        .is_file()
    {
        err(ERR_SRC);
    }
    if std::fs::metadata(&dest)
        .unwrap_or_else(|_| err(ERR_DEST))
        .is_file()
    {
        err(ERR_DEST);
    }

    let flags = [args.update, args.soft, args.mixed, args.hard];
    if flags.iter().filter(|f| **f).count() > 1 {
        err("can only use one of 'update' , 'soft' , 'mixed' and 'hard' flags");
    }

    let mode = if args.hard {
        SyncMode::Hard
    } else if args.soft {
        SyncMode::Soft
    } else if args.update {
        SyncMode::Update
    } else {
        SyncMode::Mixed
    };

    if let Err(index) = sync_dirs(&src, &dest, args.verbose, mode) {
        if index == 1 {
            err(ERR_SRC);
        } else {
            err(ERR_DEST);
        }
    }
}
