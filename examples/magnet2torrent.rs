use std::time::Duration;

use clap::Parser;
use rbtorrent::{AlertCategory, SessionBuilder, TorrentFlags};
use rbtorrent_sys::ffi;

#[derive(Parser)]
struct Opt {
    magnet_uri: String,
}

fn main() {
    let opt = Opt::parse();

    let session = SessionBuilder::new()
        .with_alert_mask(AlertCategory::STATUS | AlertCategory::ERROR)
        .set_disable_disk(true)
        .build();

    let mut atp = rbtorrent::AddTorrentParams::from_magnet_uri(opt.magnet_uri);
    atp.save_path = Some(".".parse().unwrap());
    atp.torrent_flags = Some(TorrentFlags::DEFAULT_DONT_DOWNLOAD);

    session.add_torrent(atp);

    loop {
        session.handle_alerts(|alerts| {
            for mut alert in alerts {
                println!("{}", alert.message());

                if let Some(alert) = alert.as_metadata_received() {
                    let handle = alert.torrent_handle();
                } else if let Some(alert) = alert.as_save_resume_data() {
                } else if let Some(alert) = alert.as_save_resume_data_failed() {
                }
            }
        });

        // 	for (lt::alert* a : alerts)
        // 	{
        // 		std::cout << a->message() << std::endl;
        // 		if (auto const* mra = lt::alert_cast<lt::metadata_received_alert>(a))
        // 		{
        // 			std::cerr << "metadata received" << std::endl;
        // 			auto const handle = mra->handle;
        // 			std::shared_ptr<lt::torrent_info const> ti = handle.torrent_file();
        // 			if (!ti)
        // 			{
        // 				std::cerr << "unexpected missing torrent info" << std::endl;
        // 				goto done;
        // 			}

        // 			// in order to create valid v2 torrents, we need to download the
        // 			// piece hashes. libtorrent currently only downloads the hashes
        // 			// on-demand, so we would have to download all the content.
        // 			// Instead, produce an invalid v2 torrent that's missing piece
        // 			// layers
        // 			if (ti->v2())
        // 			{
        // 				std::cerr << "found v2 torrent, generating a torrent missing piece hashes" << std::endl;
        // 			}
        // 			handle.save_resume_data(lt::torrent_handle::save_info_dict);
        // 			handle.set_flags(lt::torrent_flags::paused);
        // 		}
        // 		else if (auto* rda = lt::alert_cast<lt::save_resume_data_alert>(a))
        // 		{
        // 			// don't include piece layers
        // 			rda->params.merkle_trees.clear();
        // 			lt::entry e = lt::write_torrent_file(rda->params, lt::write_flags::allow_missing_piece_layer);
        // 			std::vector<char> torrent;
        // 			lt::bencode(std::back_inserter(torrent), e);
        // 			std::ofstream out(argv[2]);
        // 			out.write(torrent.data(), int(torrent.size()));
        // 			goto done;
        // 		}
        // 		else if (auto const* rdf = lt::alert_cast<lt::save_resume_data_failed_alert>(a))
        // 		{
        // 			std::cerr << "failed to save resume data: " << rdf->message() << std::endl;
        // 			goto done;
        // 		}
        // 	}
        session.wait_for_alert(Duration::from_millis(200));
    }
}
