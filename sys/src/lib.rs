#![deny(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

#[cxx::bridge(namespace = "libtorrent")]
pub mod ffi {
    #[repr(i32)]
    enum torrent_state {
        queued_for_checking,
        checking_files,
        downloading_metadata,
        downloading,
        finished,
        seeding,
        allocating,
        checking_resume_data,
    }

    struct TorrentStatus {
        state: torrent_state,
        progress_ppm: u64,
    }

    struct TorrentInfoNode {
        hostname: String,
        port: u16,
    }

    #[derive(Debug)]
    struct AnnounceEntry {
        /// tracker URL as it appeared in the torrent file
        url: String,

        /// the current ``&trackerid=`` argument passed to the tracker.
        /// this is optional and is normally empty (in which case no
        /// trackerid is sent).
        trackerid: String,

        /// the tier this tracker belongs to
        tier: u8,
    }

    #[derive(Copy, Clone)]
    struct Alert {
        inner: *mut alert,
    }

    unsafe extern "C++" {
        include!("src/rb.hpp");

        type alert;
        type session;
        type session_params;
        type add_torrent_params;
        type metadata_received_alert;
        type save_resume_data_alert;
        type save_resume_data_failed_alert;
        type torrent_handle;
        type torrent_info;
        type torrent_state;

        pub fn new_session_params() -> UniquePtr<session_params>;

        pub fn session_params_set_user_agent(
            session_params: Pin<&mut session_params>,
            user_agent: &str,
        );

        pub fn session_params_set_outgoing_interfaces(
            session_params: Pin<&mut session_params>,
            outgoing_interfaces: &str,
        );

        pub fn session_params_set_listen_interfaces(
            session_params: Pin<&mut session_params>,
            listen_interfaces: &str,
        );

        pub fn session_params_set_alert_mask(
            session_params: Pin<&mut session_params>,
            alert_mask: i32,
        );

        pub fn session_params_disable_disk(session_params: Pin<&mut session_params>);

        /// This function return a struct of type lt::session
        pub fn new_session(settings: UniquePtr<session_params>) -> UniquePtr<session>;

        /// This function return a struct of type lt::add_torrent_params
        ///
        /// lt::add_torrent_params is returned by lt::parse_magnet_uri,
        pub fn new_add_torrent_params_from_magnet_uri(uri: &str) -> UniquePtr<add_torrent_params>;

        pub fn new_add_torrent_params_from_torrent_file(
            path: &str,
        ) -> UniquePtr<add_torrent_params>;

        pub fn add_torrent_params_set_save_path(params: Pin<&mut add_torrent_params>, path: &str);

        pub fn add_torrent_params_set_trackers(
            params: Pin<&mut add_torrent_params>,
            trackers: &[&str],
        );

        pub fn add_torrent_params_set_torrent_flags(
            params: Pin<&mut add_torrent_params>,
            torrent_flags: u64,
        );

        /// This function return a struct of type lt::torrent_handle
        ///
        /// Call the function add_torrent() using the current session and add_torrent_params
        /// given in parameters
        pub fn session_add_torrent(
            ses: Pin<&mut session>,
            params: Pin<&mut add_torrent_params>,
        ) -> UniquePtr<torrent_handle>;

        /// This function remove the given torrent from session
        pub fn session_remove_torrent(ses: Pin<&mut session>, hdl: &torrent_handle);

        /// This function call pause() for the given session
        pub fn session_pause(ses: Pin<&mut session>);

        pub fn session_get_alerts(ses: Pin<&mut session>) -> Vec<Alert>;

        // Alert
        pub fn alert_message(alert: &Alert) -> String;

        pub fn alert_cast_metadata_received(alert: Alert) -> *mut metadata_received_alert;

        pub unsafe fn metadata_received_alert_get_torrent_handle(
            alert: *mut metadata_received_alert,
        ) -> *mut torrent_handle;

        pub fn alert_cast_save_resume_data(alert: Alert) -> *mut save_resume_data_alert;

        pub fn alert_cast_save_resume_data_failed(
            alert: Alert,
        ) -> *mut save_resume_data_failed_alert;

        pub fn wait_for_alert(ses: Pin<&mut session>, duration_ms: u64);

        /// This function return true if torrent has metadata
        pub fn torrent_has_metadata(hdl: &torrent_handle) -> bool;

        /// This function return the torrent's name
        ///
        /// Name is found in torrent_info return by function torrent_file()
        pub fn torrent_get_name(hdl: &torrent_handle) -> &str;

        pub fn torrent_get_status(hdl: &torrent_handle) -> TorrentStatus;

        /// This function return bencoded data by lt::bencode()
        pub fn torrent_bencode(hdl: &torrent_handle) -> &[u8];

        pub fn create_torrent_for_path(path: &str) -> Vec<u8>;

        pub fn open_torrent_info(path: &str) -> UniquePtr<torrent_info>;

        pub fn torrent_info_nodes(info: &torrent_info) -> Vec<TorrentInfoNode>;

        pub fn torrent_info_trackers(info: &torrent_info) -> Vec<AnnounceEntry>;

        /// This function call libtorrent::version() and return libtorrent version
        pub fn version() -> *const c_char;
    }
}
