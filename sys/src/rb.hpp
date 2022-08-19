#pragma once

#include "rust/cxx.h"

#include "libtorrent/alert_types.hpp"
#include "libtorrent/create_torrent.hpp"
#include "libtorrent/load_torrent.hpp"
#include "libtorrent/magnet_uri.hpp"
#include "libtorrent/session.hpp"
#include "libtorrent/session_params.hpp"
#include "libtorrent/torrent_status.hpp"
#include "libtorrent/version.hpp"

#include <memory>

char const* version();

namespace libtorrent {
    using torrent_state = lt::torrent_status::state_t;

    struct Alert;
    struct AnnounceEntry;
    struct TorrentInfoNode;
    struct TorrentStatus;

    std::unique_ptr<session_params> new_session_params();
    void session_params_set_user_agent(session_params& params, rust::Str user_agent);
    void session_params_set_outgoing_interfaces(session_params& params, rust::Str outgoing_interfaces);
    void session_params_set_listen_interfaces(session_params& params, rust::Str listen_interfaces);
    void session_params_set_alert_mask(session_params& params, int alert_mask);
    void session_params_disable_disk(session_params& params);
    std::unique_ptr<session> new_session(std::unique_ptr<session_params> params);
    std::unique_ptr<add_torrent_params> new_add_torrent_params_from_magnet_uri(rust::Str uri);
    std::unique_ptr<add_torrent_params> new_add_torrent_params_from_torrent_file(rust::Str path);
    void add_torrent_params_set_save_path(add_torrent_params& params, rust::Str path);
    void add_torrent_params_set_trackers(add_torrent_params& params, rust::Slice<const rust::Str> trackers);
    void add_torrent_params_set_torrent_flags(add_torrent_params& params, uint64_t torrent_flags);
    std::unique_ptr<lt::torrent_handle> session_add_torrent(session& ses, add_torrent_params& params);
    void session_remove_torrent(session& ses, const torrent_handle& hdl);
    void session_pause(session& ses);
    rust::Vec<Alert> session_get_alerts(session& ses);
    rust::String alert_message(const Alert& alert);
    metadata_received_alert* alert_cast_metadata_received(Alert alert);
    torrent_handle* metadata_received_alert_get_torrent_handle(metadata_received_alert* alert);
    save_resume_data_alert* alert_cast_save_resume_data(Alert alert);
    save_resume_data_failed_alert* alert_cast_save_resume_data_failed(Alert alert);
    void wait_for_alert(session& ses, uint64_t max_duration);
    bool torrent_has_metadata(const torrent_handle& hdl);
    rust::Str torrent_get_name(const torrent_handle& hdl);
    TorrentStatus torrent_get_status(const torrent_handle& hdl);
    rust::Slice<const uint8_t> torrent_bencode(const torrent_handle& hdl);
    rust::Vec<uint8_t> create_torrent_for_path(rust::Str path);
    std::unique_ptr<torrent_info> open_torrent_info(rust::Str path);
    rust::Vec<TorrentInfoNode> torrent_info_nodes(const torrent_info& info);
    rust::Vec<AnnounceEntry> torrent_info_trackers(const torrent_info& info);
}
