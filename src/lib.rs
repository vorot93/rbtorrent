#![doc = include_str!("../README.md")]

use bitflags::bitflags;
use cxx::UniquePtr;
pub use rbtorrent_sys::ffi::{torrent_info, AnnounceEntry, TorrentStatus};
pub use rbtorrent_sys::*;
use std::{
    collections::HashMap,
    marker::PhantomData,
    path::PathBuf,
    sync::mpsc::{channel, Sender},
    time::Duration,
};

#[derive(Debug)]
pub enum AddTorrentParamsSource {
    Magnet(String),
    Torrent(PathBuf),
}

#[derive(Debug)]
pub struct AddTorrentParams {
    pub source: AddTorrentParamsSource,
    pub save_path: Option<PathBuf>,
    pub trackers: Option<Vec<String>>,
    pub torrent_flags: Option<TorrentFlags>,
}

impl AddTorrentParams {
    pub fn from_magnet_uri(magnet_uri: String) -> Self {
        Self {
            source: AddTorrentParamsSource::Magnet(magnet_uri),
            save_path: None,
            trackers: None,
            torrent_flags: None,
        }
    }
}

#[derive(Debug)]
pub struct TorrentHandle {
    cmd_tx: Sender<SessionCommand>,
    i: usize,
}

struct DirectTorrentHandle<'a> {
    inner: *mut ffi::torrent_handle,
    _marker: PhantomData<&'a ()>,
}

pub trait TorrentHandleTrait {
    fn get_name(&self) -> Option<String>;
    fn get_status(&self) -> Option<TorrentStatus>;
}

impl<'a> TorrentHandleTrait for DirectTorrentHandle<'a> {
    fn get_name(&self) -> Option<String> {
        Some(ffi::torrent_get_name(unsafe { &*self.inner }).to_string())
    }

    fn get_status(&self) -> Option<TorrentStatus> {
        Some(ffi::torrent_get_status(unsafe { &*self.inner }))
    }
}

pub enum SessionCommand {
    AddTorrent {
        params: AddTorrentParams,
        cb: Sender<TorrentHandle>,
    },
    RemoveTorrent {
        handle: TorrentHandle,
        cb: Sender<()>,
    },
    HandleAlerts {
        f: Box<dyn FnOnce(Vec<Alert>) + Send + Sync + 'static>,
        cb: Sender<()>,
    },
    WaitForAlert {
        max_duration: Duration,
        cb: Sender<()>,
    },
    GetTorrentName {
        i: usize,
        cb: Sender<String>,
    },
    GetTorrentStatus {
        i: usize,
        cb: Sender<TorrentStatus>,
    },
}

pub struct Alert(ffi::Alert);

pub struct MetadataReceivedAlert<'alert> {
    inner: *mut ffi::metadata_received_alert,
    _marker: PhantomData<&'alert mut ()>,
}

impl<'alert> MetadataReceivedAlert<'alert> {
    pub fn torrent_handle(&self) -> impl TorrentHandleTrait + '_ {
        DirectTorrentHandle {
            inner: unsafe { ffi::metadata_received_alert_get_torrent_handle(self.inner) },
            _marker: PhantomData,
        }
    }
}

pub struct SaveResumeDataAlert<'alert> {
    inner: *mut ffi::save_resume_data_alert,
    _marker: PhantomData<&'alert mut ()>,
}

pub struct SaveResumeDataFailedAlert<'alert> {
    inner: *mut ffi::save_resume_data_failed_alert,
    _marker: PhantomData<&'alert mut ()>,
}

impl Alert {
    pub fn message(&self) -> String {
        ffi::alert_message(&self.0)
    }

    pub fn as_metadata_received(&mut self) -> Option<MetadataReceivedAlert<'_>> {
        let v = ffi::alert_cast_metadata_received(self.0);

        if v.is_null() {
            None
        } else {
            Some(MetadataReceivedAlert {
                inner: v,
                _marker: PhantomData,
            })
        }
    }

    pub fn as_save_resume_data(&mut self) -> Option<SaveResumeDataAlert<'_>> {
        let v = ffi::alert_cast_save_resume_data(self.0);

        if v.is_null() {
            None
        } else {
            Some(SaveResumeDataAlert {
                inner: v,
                _marker: PhantomData,
            })
        }
    }

    pub fn as_save_resume_data_failed(&mut self) -> Option<SaveResumeDataFailedAlert<'_>> {
        let v = ffi::alert_cast_save_resume_data_failed(self.0);

        if v.is_null() {
            None
        } else {
            Some(SaveResumeDataFailedAlert {
                inner: v,
                _marker: PhantomData,
            })
        }
    }
}

pub struct Session {
    cmd_tx: Sender<SessionCommand>,
}

bitflags! {
    pub struct AlertCategory: u32 {
        /// Enables alerts that report an error. This includes:
        ///
        /// * tracker errors
        /// * tracker warnings
        /// * file errors
        /// * resume data failures
        /// * web seed errors
        /// * .torrent files errors
        /// * listen socket errors
        /// * port mapping errors
        const ERROR = 1;

        // Enables alerts when peers send invalid requests, get banned or
        // snubbed.
        const PEER = 1;

        // Enables alerts for port mapping events. For NAT-PMP and UPnP.
        const PORT_MAPPING = 2;

        // Enables alerts for events related to the storage. File errors and
        // synchronization events for moving the storage, renaming files etc.
        const STORAGE = 3;

        // Enables all tracker events. Includes announcing to trackers,
        // receiving responses, warnings and errors.
        const TRACKER = 4;

        // Low level alerts for when peers are connected and disconnected.
        const CONNECT = 5;

        // Enables alerts for when a torrent or the session changes state.
        const STATUS = 6;

        // Alerts when a peer is blocked by the ip blocker or port blocker.
        const IP_BLOCK = 8;

        // Alerts when some limit is reached that might limit the download
        // or upload rate.
        const PERFORMANCE_WARNING = 9;

        // Alerts on events in the DHT node. For incoming searches or
        // bootstrapping being done etc.
        const DHT = 10;

        // If you enable these alerts, you will receive a stats_alert
        // approximately once every second, for every active torrent.
        // These alerts contain all statistics counters for the interval since
        // the lasts stats alert.
        const STATS = 11;

        // Enables debug logging alerts. These are available unless libtorrent
        // was built with logging disabled (``TORRENT_DISABLE_LOGGING``). The
        // alerts being posted are log_alert and are session wide.
        const SESSION_LOG = 13;

        // Enables debug logging alerts for torrents. These are available
        // unless libtorrent was built with logging disabled
        // (``TORRENT_DISABLE_LOGGING``). The alerts being posted are
        // torrent_log_alert and are torrent wide debug events.
        const TORRENT_LOG = 14;

        // Enables debug logging alerts for peers. These are available unless
        // libtorrent was built with logging disabled
        // (``TORRENT_DISABLE_LOGGING``). The alerts being posted are
        // peer_log_alert and low-level peer events and messages.
        const PEER_LOG = 15;

        // enables the incoming_request_alert.
        const INCOMING_REQUEST = 16;

        // enables dht_log_alert, debug logging for the DHT
        const DHT_LOG = 17;

        // enable events from pure dht operations not related to torrents
        const DHT_OPERATION = 18;

        // enables port mapping log events. This log is useful
        // for debugging the UPnP or NAT-PMP implementation
        const PORT_MAPPING_LOG = 19;

        // enables verbose logging from the piece picker.
        const PICKER_LOG = 20;

        // alerts when files complete downloading
        const FILE_PROGRESS = 21;

        // alerts when pieces complete downloading or fail hash check
        const PIECE_PROGRESS = 22;

        // alerts when we upload blocks to other peers
        const UPLOAD = 23;

        // alerts on individual blocks being requested, downloading, finished,
        // rejected, time-out and cancelled. This is likely to post alerts at a
        // high rate.
        const BLOCK_PROGRESS = 24;
    }
}

bitflags! {
    pub struct TorrentFlags : u64 {
        // If ``seed_mode`` is set, libtorrent will assume that all files
        // are present for this torrent and that they all match the hashes in
        // the torrent file. Each time a peer requests to download a block,
        // the piece is verified against the hash, unless it has been verified
        // already. If a hash fails, the torrent will automatically leave the
        // seed mode and recheck all the files. The use case for this mode is
        // if a torrent is created and seeded, or if the user already know
        // that the files are complete, this is a way to avoid the initial
        // file checks, and significantly reduce the startup time.
        //
        // Setting ``seed_mode`` on a torrent without metadata (a
        // .torrent file) is a no-op and will be ignored.
        //
        // It is not possible to *set* the ``seed_mode`` flag on a torrent after it has
        // been added to a session. It is possible to *clear* it though.
        const SEED_MODE = 0;

        // If ``upload_mode`` is set, the torrent will be initialized in
        // upload-mode, which means it will not make any piece requests. This
        // state is typically entered on disk I/O errors, and if the torrent
        // is also auto managed, it will be taken out of this state
        // periodically (see ``settings_pack::optimistic_disk_retry``).
        //
        // This mode can be used to avoid race conditions when
        // adjusting priorities of pieces before allowing the torrent to start
        // downloading.
        //
        // If the torrent is auto-managed (``auto_managed``), the torrent
        // will eventually be taken out of upload-mode, regardless of how it
        // got there. If it's important to manually control when the torrent
        // leaves upload mode, don't make it auto managed.
        const UPLOAD_MODE = 1;

        // determines if the torrent should be added in *share mode* or not.
        // Share mode indicates that we are not interested in downloading the
        // torrent, but merely want to improve our share ratio (i.e. increase
        // it). A torrent started in share mode will do its best to never
        // download more than it uploads to the swarm. If the swarm does not
        // have enough demand for upload capacity, the torrent will not
        // download anything. This mode is intended to be safe to add any
        // number of torrents to, without manual screening, without the risk
        // of downloading more than is uploaded.
        //
        // A torrent in share mode sets the priority to all pieces to 0,
        // except for the pieces that are downloaded, when pieces are decided
        // to be downloaded. This affects the progress bar, which might be set
        // to "100% finished" most of the time. Do not change file or piece
        // priorities for torrents in share mode, it will make it not work.
        //
        // The share mode has one setting, the share ratio target, see
        // ``settings_pack::share_mode_target`` for more info.
        const SHARE_MODE = 2;

        // determines if the IP filter should apply to this torrent or not. By
        // default all torrents are subject to filtering by the IP filter
        // (i.e. this flag is set by default). This is useful if certain
        // torrents needs to be exempt for some reason, being an auto-update
        // torrent for instance.
        const APPLY_IP_FILTER = 3;

        // specifies whether or not the torrent is paused. i.e. it won't connect to the tracker or any of the peers
        // until it's resumed. Note that a paused torrent that also has the
        // auto_managed flag set can be started at any time by libtorrent's queuing
        // logic. See queuing_.
        const PAUSED = 4;

        // If the torrent is auto-managed (``auto_managed``), the torrent
        // may be resumed at any point, regardless of how it paused. If it's
        // important to manually control when the torrent is paused and
        // resumed, don't make it auto managed.
        //
        // If ``auto_managed`` is set, the torrent will be queued,
        // started and seeded automatically by libtorrent. When this is set,
        // the torrent should also be started as paused. The default queue
        // order is the order the torrents were added. They are all downloaded
        // in that order. For more details, see queuing_.
        const AUTO_MANAGED = 5;

        // used in add_torrent_params to indicate that it's an error to attempt
        // to add a torrent that's already in the session. If it's not considered an
        // error, a handle to the existing torrent is returned.
        // This flag is not saved by write_resume_data(), since it is only meant for
        // adding torrents.
        const DUPLICATE_IS_ERROR = 6;

        // on by default and means that this torrent will be part of state
        // updates when calling post_torrent_updates().
        // This flag is not saved by write_resume_data().
        const UPDATE_SUBSCRIBE = 7;

        // sets the torrent into super seeding/initial seeding mode. If the torrent
        // is not a seed, this flag has no effect.
        const SUPER_SEEDING = 8;

        // sets the sequential download state for the torrent. In this mode the
        // piece picker will pick pieces with low index numbers before pieces with
        // high indices. The actual pieces that are picked depend on other factors
        // still, such as which pieces a peer has and whether it is in parole mode
        // or "prefer whole pieces"-mode. Sequential mode is not ideal for streaming
        // media. For that, see set_piece_deadline() instead.
        const SEQUENTIAL_DOWNLOAD = 9;

        // When this flag is set, the torrent will *force stop* whenever it
        // transitions from a non-data-transferring state into a data-transferring
        // state (referred to as being ready to download or seed). This is useful
        // for torrents that should not start downloading or seeding yet, but want
        // to be made ready to do so. A torrent may need to have its files checked
        // for instance, so it needs to be started and possibly queued for checking
        // (auto-managed and started) but as soon as it's done, it should be
        // stopped.
        //
        // *Force stopped* means auto-managed is set to false and it's paused. As
        // if the auto_manages flag is cleared and the paused flag is set on the torrent.
        //
        // Note that the torrent may transition into a downloading state while
        // setting this flag, and since the logic is edge triggered you may
        // miss the edge. To avoid this race, if the torrent already is in a
        // downloading state when this call is made, it will trigger the
        // stop-when-ready immediately.
        //
        // When the stop-when-ready logic fires, the flag is cleared. Any
        // subsequent transitions between downloading and non-downloading states
        // will not be affected, until this flag is set again.
        //
        // The behavior is more robust when setting this flag as part of adding
        // the torrent. See add_torrent_params.
        //
        // The stop-when-ready flag fixes the inherent race condition of waiting
        // for the state_changed_alert and then call pause(). The download/seeding
        // will most likely start in between posting the alert and receiving the
        // call to pause.
        //
        // A downloading state is one where peers are being connected. Which means
        // just downloading the metadata via the ``ut_metadata`` extension counts
        // as a downloading state. In order to stop a torrent once the metadata
        // has been downloaded, instead set all file priorities to dont_download
        const STOP_WHEN_READY = 10;

        // when this flag is set, the tracker list in the add_torrent_params
        // object override any trackers from the torrent file. If the flag is
        // not set, the trackers from the add_torrent_params object will be
        // added to the list of trackers used by the torrent.
        // This flag is set by read_resume_data() if there are trackers present in
        // the resume data file. This effectively makes the trackers saved in the
        // resume data take precedence over the original trackers. This includes if
        // there's an empty list of trackers, to support the case where they were
        // explicitly removed in the previous session.
        // This flag is not saved by write_resume_data()
        const OVERRIDE_TRACKERS = 11;

        // If this flag is set, the web seeds from the add_torrent_params
        // object will override any web seeds in the torrent file. If it's not
        // set, web seeds in the add_torrent_params object will be added to the
        // list of web seeds used by the torrent.
        // This flag is set by read_resume_data() if there are web seeds present in
        // the resume data file. This effectively makes the web seeds saved in the
        // resume data take precedence over the original ones. This includes if
        // there's an empty list of web seeds, to support the case where they were
        // explicitly removed in the previous session.
        // This flag is not saved by write_resume_data()
        const OVERRIDE_WEB_SEEDS = 12;

        // if this flag is set (which it is by default) the torrent will be
        // considered needing to save its resume data immediately as it's
        // added. New torrents that don't have any resume data should do that.
        // This flag is cleared by a successful call to save_resume_data()
        // This flag is not saved by write_resume_data(), since it represents an
        // ephemeral state of a running torrent.
        const NEED_SAVE_RESUME = 13;

        // set this flag to disable DHT for this torrent. This lets you have the DHT
        // enabled for the whole client, and still have specific torrents not
        // participating in it. i.e. not announcing to the DHT nor picking up peers
        // from it.
        const DISABLE_DHT = 19;

        // set this flag to disable local service discovery for this torrent.
        const DISABLE_LSD = 20;

        // set this flag to disable peer exchange for this torrent.
        const DISABLE_PEX = 21;

        // if this flag is set, the resume data will be assumed to be correct
        // without validating it against any files on disk. This may be used when
        // restoring a session by loading resume data from disk. It will save time
        // and also delay any hard disk errors until files are actually needed. If
        // the resume data cannot be trusted, or if a torrent is added for the first
        // time to some save path that may already have some of the files, this flag
        // should not be set.
        const NO_VERIFY_FILES = 22;

        // default all file priorities to dont_download. This is useful for adding
        // magnet links where the number of files is unknown, but the
        // file_priorities is still set for some files. Any file not covered by
        // the file_priorities list will be set to normal download priority,
        // unless this flag is set, in which case they will be set to 0
        // (dont_download).
        const DEFAULT_DONT_DOWNLOAD = 23;
    }
}

#[derive(Debug, Default)]
pub struct SessionBuilder {
    user_agent: Option<String>,
    outgoing_interfaces: Option<Vec<String>>,
    listen_interfaces: Option<Vec<String>>,
    alert_mask: Option<AlertCategory>,
    disable_disk: bool,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_outgoing_interfaces(mut self, outgoing_interfaces: Vec<String>) -> Self {
        self.outgoing_interfaces = Some(outgoing_interfaces);
        self
    }

    pub fn with_listen_interfaces(mut self, listen_interfaces: Vec<String>) -> Self {
        self.listen_interfaces = Some(listen_interfaces);
        self
    }

    pub fn with_alert_mask(mut self, alert_mask: AlertCategory) -> Self {
        self.alert_mask = Some(alert_mask);
        self
    }

    pub fn set_disable_disk(mut self, doit: bool) -> Self {
        self.disable_disk = doit;
        self
    }

    pub fn build(self) -> Session {
        let (cmd_tx, cmd_rx) = channel();

        let (started_tx, started_rx) = channel();

        std::thread::spawn({
            let cmd_tx = cmd_tx.clone();
            move || {
                let mut params = ffi::new_session_params();

                if let Some(v) = self.user_agent {
                    ffi::session_params_set_user_agent(params.pin_mut(), &v);
                }

                if let Some(v) = self.outgoing_interfaces {
                    let v = v.join(",");
                    ffi::session_params_set_outgoing_interfaces(params.pin_mut(), &v);
                }

                if let Some(v) = self.listen_interfaces {
                    let v = v.join(",");
                    ffi::session_params_set_listen_interfaces(params.pin_mut(), &v);
                }

                if let Some(v) = self.alert_mask {
                    ffi::session_params_set_alert_mask(params.pin_mut(), v.bits() as i32);
                }

                if self.disable_disk {
                    ffi::session_params_disable_disk(params.pin_mut());
                }

                let mut session = ffi::new_session(params);

                let _ = started_tx.send(());

                let mut add_torrent_counter = 0_usize;

                let mut added_torrents = HashMap::<usize, UniquePtr<ffi::torrent_handle>>::new();

                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        SessionCommand::AddTorrent { params, cb } => {
                            let mut p = match params.source {
                                AddTorrentParamsSource::Magnet(magnet) => {
                                    ffi::new_add_torrent_params_from_magnet_uri(&magnet)
                                }
                                AddTorrentParamsSource::Torrent(path) => {
                                    let path = path.to_string_lossy();
                                    ffi::new_add_torrent_params_from_torrent_file(&path)
                                }
                            };

                            if let Some(v) = params.save_path {
                                let v = v.to_string_lossy();
                                ffi::add_torrent_params_set_save_path(p.pin_mut(), &v);
                            }

                            if let Some(v) = params.trackers {
                                ffi::add_torrent_params_set_trackers(
                                    p.pin_mut(),
                                    &v.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
                                );
                            }

                            let handle = ffi::session_add_torrent(session.pin_mut(), p.pin_mut());

                            added_torrents.insert(add_torrent_counter, handle);

                            let torrent_handle = TorrentHandle {
                                cmd_tx: cmd_tx.clone(),
                                i: add_torrent_counter,
                            };

                            add_torrent_counter += 1;

                            let _ = cb.send(torrent_handle);
                        }
                        SessionCommand::RemoveTorrent { handle, cb } => {
                            let hdl = added_torrents.remove(&handle.i).unwrap();
                            ffi::session_remove_torrent(session.pin_mut(), &hdl);

                            let _ = cb.send(());
                        }
                        SessionCommand::HandleAlerts { f, cb } => {
                            let alerts = ffi::session_get_alerts(session.pin_mut())
                                .into_iter()
                                .map(Alert)
                                .collect();
                            (f)(alerts);

                            let _ = cb.send(());
                        }
                        SessionCommand::WaitForAlert { max_duration, cb } => {
                            ffi::wait_for_alert(session.pin_mut(), max_duration.as_millis() as u64);

                            let _ = cb.send(());
                        }
                        SessionCommand::GetTorrentName { i, cb } => {
                            let _ = cb.send(
                                ffi::torrent_get_name(added_torrents.get(&i).unwrap()).to_string(),
                            );
                        }
                        SessionCommand::GetTorrentStatus { i, cb } => {
                            let _ =
                                cb.send(ffi::torrent_get_status(added_torrents.get(&i).unwrap()));
                        }
                    }
                }
            }
        });

        started_rx.recv().unwrap();

        Session { cmd_tx }
    }
}

impl Session {
    pub fn add_torrent(&self, params: AddTorrentParams) -> TorrentHandle {
        let (cb_tx, cb_rx) = channel();
        let _ = self
            .cmd_tx
            .send(SessionCommand::AddTorrent { params, cb: cb_tx });
        cb_rx.recv().unwrap()
    }

    pub fn remove_torrent(&self, handle: TorrentHandle) {
        let (cb_tx, cb_rx) = channel();
        let _ = self
            .cmd_tx
            .send(SessionCommand::RemoveTorrent { handle, cb: cb_tx });
        cb_rx.recv().unwrap()
    }

    pub fn handle_alerts(&self, f: impl FnOnce(Vec<Alert>) + Send + Sync + 'static) {
        let (cb_tx, cb_rx) = channel();
        let _ = self.cmd_tx.send(SessionCommand::HandleAlerts {
            f: Box::new(f),
            cb: cb_tx,
        });
        cb_rx.recv().unwrap()
    }

    pub fn wait_for_alert(&self, max_duration: Duration) {
        let (cb_tx, cb_rx) = channel();
        let _ = self.cmd_tx.send(SessionCommand::WaitForAlert {
            max_duration,
            cb: cb_tx,
        });
        cb_rx.recv().unwrap()
    }
}

impl TorrentHandleTrait for TorrentHandle {
    fn get_name(&self) -> Option<String> {
        let (cb_tx, cb_rx) = channel();
        if self
            .cmd_tx
            .send(SessionCommand::GetTorrentName {
                i: self.i,
                cb: cb_tx,
            })
            .is_ok()
        {
            Some(cb_rx.recv().unwrap())
        } else {
            None
        }
    }

    fn get_status(&self) -> Option<TorrentStatus> {
        let (cb_tx, cb_rx) = channel();
        if self
            .cmd_tx
            .send(SessionCommand::GetTorrentStatus {
                i: self.i,
                cb: cb_tx,
            })
            .is_ok()
        {
            Some(cb_rx.recv().unwrap())
        } else {
            None
        }
    }
}

pub struct TorrentInfo {
    inner: UniquePtr<torrent_info>,
}

impl TorrentInfo {
    pub fn open(path: &str) -> Self {
        Self {
            inner: ffi::open_torrent_info(path),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = (String, u16)> {
        ffi::torrent_info_nodes(&self.inner)
            .into_iter()
            .map(|inner| (inner.hostname, inner.port))
    }

    pub fn trackers(&self) -> Vec<AnnounceEntry> {
        ffi::torrent_info_trackers(&self.inner)
    }
}
