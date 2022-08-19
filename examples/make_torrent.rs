use clap::Parser;
use expanded_pathbuf::ExpandedPathBuf;
use std::io::Write;

/// Generates a torrent file from the specified file or directory and writes it to standard out
#[derive(Debug, Parser)]
struct Opt {
    file: ExpandedPathBuf,
    /// Specifies the output filename of the torrent file. If this is not specified, the torrent file is printed to the standard out.
    #[clap(short, long)]
    output: Option<ExpandedPathBuf>,
}

fn main() {
    let opt = Opt::parse();

    let encoded = rbtorrent::ffi::create_torrent_for_path(&opt.file.0.to_string_lossy());

    if let Some(output) = opt.output {
        std::fs::write(output.0, encoded).unwrap();
    } else {
        std::io::stdout().write_all(&encoded).unwrap();
    }
}
