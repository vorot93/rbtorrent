#include "src/rb.hpp"
#include "rbtorrent-sys/src/lib.rs.h"
#include "libtorrent/disabled_disk_io.hpp"
#include <iterator>

namespace libtorrent {

std::unique_ptr<session_params> new_session_params() {
	return std::make_unique<session_params>();
}
void session_params_set_user_agent(session_params& params, rust::Str user_agent) {
	params.settings.set_str(settings_pack::user_agent, std::string(user_agent));
}

void session_params_set_outgoing_interfaces(session_params& params, rust::Str outgoing_interfaces) {
	params.settings.set_str(settings_pack::outgoing_interfaces, std::string(outgoing_interfaces));
}

void session_params_set_listen_interfaces(session_params& params, rust::Str listen_interfaces) {
	params.settings.set_str(settings_pack::listen_interfaces, std::string(listen_interfaces));
}

void session_params_set_alert_mask(session_params& params, int alert_mask) {
	params.settings.set_int(settings_pack::alert_mask, alert_mask);
}

void session_params_disable_disk(session_params& params) {
	params.disk_io_constructor = disabled_disk_io_constructor;
}

std::unique_ptr<session> new_session(std::unique_ptr<session_params> params) {
	return std::make_unique<session>(std::move(*params));
}

std::unique_ptr<add_torrent_params> new_add_torrent_params_from_magnet_uri(rust::Str uri) {
	std::string s(uri);
	add_torrent_params p;

	p = parse_magnet_uri(s.c_str());

	return std::make_unique<add_torrent_params>(std::move(p));
}

std::unique_ptr<add_torrent_params> new_add_torrent_params_from_torrent_file(rust::Str path) {
	return std::make_unique<add_torrent_params>(load_torrent_file(std::string(path)));
}

void add_torrent_params_set_save_path(add_torrent_params& params, rust::Str path) {
	params.save_path = std::string(path);
}

void add_torrent_params_set_trackers(add_torrent_params& params, rust::Slice<const rust::Str> trackers) {
	params.trackers.clear();

	for (auto tracker : trackers) {
		params.trackers.push_back(std::string(tracker));
	}
}

void add_torrent_params_set_torrent_flags(add_torrent_params& params, uint64_t torrent_flags) {
	params.flags = torrent_flags_t(torrent_flags);
}

std::unique_ptr<torrent_handle> session_add_torrent(session& ses, add_torrent_params& params) {
	torrent_handle hdl;

	hdl = ses.add_torrent(params);

	return std::make_unique<torrent_handle>(std::move(hdl));
}

void session_remove_torrent(session& ses, const torrent_handle& hdl) {
	ses.remove_torrent(hdl);
}

void session_pause(session& ses) {
	ses.pause();
}

rust::Vec<Alert> session_get_alerts(session& ses) {
	std::vector<alert*> alerts;

	ses.pop_alerts(&alerts);

	rust::Vec<Alert> out;

	for (auto alert : alerts) {
		out.push_back(Alert { inner: alert });
	}

	return out;
}

rust::String alert_message(const Alert& alert) {
	return rust::String(alert.inner->message());
}

metadata_received_alert* alert_cast_metadata_received(Alert alert) {
	return alert_cast<metadata_received_alert>(alert.inner);
}

torrent_handle* metadata_received_alert_get_torrent_handle(metadata_received_alert* alert) {
	return &alert->handle;
}

save_resume_data_alert* alert_cast_save_resume_data(Alert alert) {
	return alert_cast<save_resume_data_alert>(alert.inner);
}

save_resume_data_failed_alert* alert_cast_save_resume_data_failed(Alert alert) {
	return alert_cast<save_resume_data_failed_alert>(alert.inner);
}

void wait_for_alert(session& ses, uint64_t max_duration) {
	ses.wait_for_alert(std::chrono::milliseconds(max_duration));
}

bool torrent_has_metadata(const torrent_handle& hdl) {
	return hdl.status().has_metadata;
}

rust::Str torrent_get_name(const torrent_handle& hdl) {
	auto infos = hdl.torrent_file();
	return infos->name();
}

TorrentStatus torrent_get_status(const torrent_handle& hdl) {
	auto s = hdl.status();

	return TorrentStatus {
		state: s.state,
		progress_ppm: uint64_t(s.progress_ppm),
	};
}

rust::Slice<const uint8_t> torrent_bencode(const torrent_handle& hdl) {
	auto infos = hdl.torrent_file();
	auto entry = create_torrent(*infos).generate();
	std::vector<char> vec;
	bencode(std::back_inserter(vec), entry);
	rust::Slice<const uint8_t> slice{reinterpret_cast<const unsigned char *>(vec.data()), vec.size()};
	return slice;
}

static std::string branch_path(std::string const& f) {
	if (f.empty()) return f;

#ifdef TORRENT_WINDOWS
	if (f == "\\\\") return "";
#endif
	if (f == "/") return "";

	auto len = f.size();
	// if the last character is / or \ ignore it
	if (f[len-1] == '/' || f[len-1] == '\\') --len;
	while (len > 0) {
		--len;
		if (f[len] == '/' || f[len] == '\\')
			break;
	}

	if (f[len] == '/' || f[len] == '\\') ++len;
	return std::string(f.c_str(), len);
}

rust::Vec<uint8_t> create_torrent_for_path(rust::Str path) {
	file_storage fs;

	auto p = std::string(path);

	add_files(fs, p);

	create_torrent t(fs);

	set_piece_hashes(t, branch_path(p));

	rust::Vec<uint8_t> torrent;
	bencode(std::back_inserter(torrent), t.generate());

	return torrent;
}

std::unique_ptr<torrent_info> open_torrent_info(rust::Str path) {
	return std::make_unique<torrent_info>(std::string(path));
}

rust::Vec<TorrentInfoNode> torrent_info_nodes(const lt::torrent_info& info) {
	rust::Vec<TorrentInfoNode> v;

	auto nodes = info.nodes();
	for (const auto& node : nodes) {
		v.push_back(TorrentInfoNode {
			hostname: rust::String(node.first),
			port: uint16_t(node.second),
		});
	}
	return v;
}

rust::Vec<AnnounceEntry> torrent_info_trackers(const lt::torrent_info& info) {
	rust::Vec<AnnounceEntry> v;

	for (const auto& e : info.trackers()) {
		v.push_back(AnnounceEntry {
			url: std::string(e.url),
			trackerid: std::string(e.trackerid),
			tier: e.tier,
		});
	}

	return v;
}

}
