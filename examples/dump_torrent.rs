use clap::Parser;
use expanded_pathbuf::ExpandedPathBuf;
use rbtorrent::{ffi::AnnounceEntry, TorrentInfo};

/// Generates a torrent file from the specified file or directory and writes it to standard out
#[derive(Debug, Parser)]
struct Opt {
    file: ExpandedPathBuf,
}

fn main() {
    let opt = Opt::parse();

    let torrent_info = TorrentInfo::open(&opt.file.to_string_lossy());

    let mut nodes_printed = false;
    for (hostname, port) in torrent_info.nodes() {
        if !nodes_printed {
            println!("nodes:");
            nodes_printed = true;
        }

        println!("{hostname}: {port}");
    }

    let mut trackers_printed = false;
    for AnnounceEntry { url, tier, .. } in torrent_info.trackers() {
        if !trackers_printed {
            println!("trackers:");
            trackers_printed = true;
        }

        println!("{tier:02}: {url}");
    }
}
