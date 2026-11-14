//! Speakers on the local network that Spotify's device list leaves out.
//!
//! `/me/player/devices` only lists devices already signed in to this account.
//! A Spotify Connect speaker waiting for an account, a Google Cast speaker
//! that has never run Spotify, and a Sonos player that is idle are all absent
//! from it. They are found here over mDNS (`crate::zeroconf`), offered in the
//! ordinary device list (`commands::connect::get_devices`), and signed in
//! when playback is moved to one: ZeroConf `addUser` for a Connect speaker,
//! Cast v2 for a Cast one (`crate::cast`), and for a Sonos, which takes no
//! credential, a transfer over the Connect state service (`crate::sonos`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::connect::ConnectDevice;
use crate::errors::AppError;
use crate::state::AppState;
use crate::zeroconf::Receiver;

/// How long one mDNS browse listens.
const BROWSE_FOR: Duration = Duration::from_secs(3);
/// A device list asked for later than this after the last scan starts a new
/// one in the background.
const RESCAN_AFTER: Duration = Duration::from_secs(60);
/// How long a receiver that accepted an account has to show up in the device
/// list before playback is moved to it.
const SIGN_IN_WAIT: Duration = Duration::from_secs(10);

/// What is known about the network, for the signed-in account.
pub struct Lan {
    found:    Mutex<Vec<Receiver>>,
    scanned:  Mutex<Option<Instant>>,
    scanning: AtomicBool,
    /// Restricted devices and the Sonos players among the receivers.
    pub connect: crate::sonos::Controller,
}

pub static LAN: LazyLock<Lan> = LazyLock::new(|| Lan {
    found:    Mutex::new(Vec::new()),
    scanned:  Mutex::new(None),
    scanning: AtomicBool::new(false),
    connect:  crate::sonos::Controller::default(),
});

impl Lan {
    fn found(&self) -> Vec<Receiver> {
        self.found.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// The receiver with a Connect device id, once it has one.
    fn receiver(&self, device_id: &str) -> Option<Receiver> {
        self.found().into_iter().find(|receiver| receiver.device_id.as_deref() == Some(device_id))
    }

    /// Forgets the network and the restricted devices, for a sign-out.
    pub fn clear(&self) {
        self.found.lock().unwrap_or_else(|p| p.into_inner()).clear();
        *self.scanned.lock().unwrap_or_else(|p| p.into_inner()) = None;
        self.connect.clear();
    }
}

/// A receiver, as the interface is told about it.
#[derive(Debug, Clone, Serialize)]
pub struct ReceiverRow {
    pub name:      String,
    pub device_id: Option<String>,
    pub address:   String,
    pub port:      u16,
    /// "connect", "cast" or "sonos".
    pub kind:      &'static str,
}

fn kind(receiver: &Receiver) -> &'static str {
    if receiver.cast {
        "cast"
    } else if crate::sonos::is_sonos(receiver) {
        "sonos"
    } else {
        "connect"
    }
}

fn rows(receivers: &[Receiver]) -> Vec<ReceiverRow> {
    receivers
        .iter()
        .map(|receiver| ReceiverRow {
            name:      receiver.name.clone(),
            device_id: receiver.device_id.clone(),
            address:   receiver.address.to_string(),
            port:      receiver.port,
            kind:      kind(receiver),
        })
        .collect()
}

/// Browses the network, then asks each Connect receiver who it is. Cast rows
/// are published as they arrive (`receivers:updated`), the rest at the end.
async fn scan(app: &AppHandle) -> Result<Vec<Receiver>, AppError> {
    if LAN.scanning.swap(true, Ordering::AcqRel) {
        return Ok(LAN.found());
    }
    let progress = app.clone();
    let result = tokio::task::spawn_blocking(move || {
        let cast_only = |found: Vec<Receiver>| {
            let mut rows_now = LAN.found();
            crate::zeroconf::merge_discovered(&mut rows_now, found);
            let _ = progress.emit("receivers:updated", rows(&rows_now));
        };
        let discovered = crate::zeroconf::discover_with_updates(BROWSE_FOR, cast_only)?;
        // A failed Connect probe still keeps the Cast rows the browse found.
        let cast: Vec<Receiver> = discovered.iter().filter(|receiver| receiver.cast).cloned().collect();
        Ok::<_, anyhow::Error>(crate::zeroconf::resolve_receivers(discovered).unwrap_or(cast))
    })
    .await;
    LAN.scanning.store(false, Ordering::Release);
    *LAN.scanned.lock().unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());
    let receivers = result
        .map_err(|e| AppError::Network(format!("network scan stopped: {e}")))?
        .map_err(|e| AppError::Network(format!("cannot browse the local network: {e:#}")))?;
    *LAN.found.lock().unwrap_or_else(|p| p.into_inner()) = receivers.clone();
    LAN.connect.remember(&receivers);
    let _ = app.emit("receivers:updated", rows(&receivers));
    Ok(receivers)
}

/// Starts a scan when the last one is old, without waiting for it.
pub fn scan_in_background(app: &AppHandle) {
    let stale = LAN
        .scanned
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .is_none_or(|at| at.elapsed() >= RESCAN_AFTER);
    if !stale || LAN.scanning.load(Ordering::Acquire) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = scan(&app).await {
            eprintln!("[receivers] {e}");
        }
    });
}

/// Rows for the receivers Spotify does not list yet, so they can be picked
/// in the ordinary device list. Moving playback to one signs it in.
pub fn waiting_rows(listed: &[ConnectDevice]) -> Vec<ConnectDevice> {
    LAN.found()
        .iter()
        .filter_map(|receiver| {
            let id = receiver.device_id.clone().filter(|id| !id.is_empty())?;
            if listed.iter().any(|device| device.id.as_deref() == Some(id.as_str()) || device.name == receiver.name) {
                return None;
            }
            crate::sonos::device_row(receiver, false).or_else(|| {
                Some(ConnectDevice {
                    id:                 Some(id),
                    is_active:          false,
                    is_private_session: false,
                    is_restricted:      false,
                    name:               receiver.name.clone(),
                    device_type:        if receiver.cast { "CastAudio" } else { "Speaker" }.into(),
                    volume_percent:     None,
                    supports_volume:    true,
                })
            })
        })
        .collect()
}

async fn our_name(app: &AppHandle) -> String {
    let db = app.state::<AppState>().db.clone();
    crate::auth::get_setting_value(&db, "device_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| crate::auth::DEFAULT_DEVICE_NAME.to_string())
}

/// Signs a receiver in to this account. A Sonos takes no account: playback is
/// moved to it over Spotify Connect instead.
async fn activate(app: &AppHandle, receiver: Receiver) -> Result<(), AppError> {
    let session = crate::internal::spclient::session(app).await?;
    if crate::sonos::is_sonos(&receiver) {
        let id = receiver.device_id.clone().ok_or_else(|| AppError::NotFound(receiver.name.clone()))?;
        return LAN.connect.transfer(&session, &id).await;
    }
    let name = receiver.name.clone();
    let epoch = crate::auth::auth_epoch();
    let result = if receiver.cast {
        let runtime = tokio::runtime::Handle::current();
        tokio::task::spawn_blocking(move || {
            crate::cast::start(receiver.address, receiver.port, &receiver.name, |client, device| {
                let token = runtime.block_on(crate::cast::device_token(&session, client, device))?;
                if !crate::auth::epoch_is_current(epoch) {
                    anyhow::bail!("the account changed before the speaker was signed in");
                }
                Ok(token)
            })
        })
        .await
    } else {
        let credentials = crate::zeroconf::Credentials::from_session(&session)
            .map_err(|e| AppError::Playback(format!("{e:#}")))?;
        let our_name = our_name(app).await;
        tokio::task::spawn_blocking(move || {
            // receivers are private lan addresses: never through a proxy
            let http = reqwest::blocking::Client::builder().no_proxy().timeout(Duration::from_secs(8)).build()?;
            let info = crate::zeroconf::get_info(&http, &receiver)?;
            if !crate::auth::epoch_is_current(epoch) {
                anyhow::bail!("the account changed before the speaker was signed in");
            }
            crate::zeroconf::add_user(&http, &receiver, &info, &credentials, &our_name)
        })
        .await
    };
    result
        .map_err(|e| AppError::Network(format!("speaker sign-in stopped: {e}")))?
        .map_err(|e| AppError::Playback(format!("{name}: {e:#}")))
}

/// Whether `device_id` is a receiver found on the network.
pub fn knows(device_id: &str) -> bool {
    LAN.receiver(device_id).is_some()
}

/// Signs in the receiver behind `device_id` and waits for Spotify to list it,
/// so a transfer to it can follow.
pub async fn sign_in_and_wait(app: &AppHandle, device_id: &str) -> Result<(), AppError> {
    let receiver = LAN.receiver(device_id).ok_or_else(|| AppError::NotFound(device_id.to_string()))?;
    let sonos = crate::sonos::is_sonos(&receiver);
    activate(app, receiver).await?;
    if sonos {
        return Ok(());
    }
    let deadline = Instant::now() + SIGN_IN_WAIT;
    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Ok(listed) = crate::commands::connect::get_devices(app.clone()).await {
            if listed.devices.iter().any(|device| device.id.as_deref() == Some(device_id)) {
                return Ok(());
            }
        }
    }
    Err(AppError::Playback("the speaker accepted the account but did not join Spotify Connect".into()))
}

/// Looks for speakers now and gives back everything found.
#[tauri::command]
pub async fn discover_receivers(app: AppHandle) -> Result<Vec<ReceiverRow>, AppError> {
    Ok(rows(&scan(&app).await?))
}

/// The speakers found by the last scan.
#[tauri::command]
pub async fn get_receivers() -> Result<Vec<ReceiverRow>, AppError> {
    Ok(rows(&LAN.found()))
}

/// Signs a speaker found on the network in to this account, so it joins
/// Spotify Connect. For a Sonos this moves playback to it.
#[tauri::command]
pub async fn activate_receiver(app: AppHandle, device_id: String) -> Result<(), AppError> {
    let receiver = LAN.receiver(&device_id).ok_or_else(|| AppError::NotFound(device_id.clone()))?;
    activate(&app, receiver).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receiver(name: &str, id: Option<&str>, port: u16, path: &str, cast: bool) -> Receiver {
        Receiver {
            name: name.into(),
            device_id: id.map(str::to_string),
            address: "192.0.2.10".parse().unwrap(),
            port,
            path: path.into(),
            cast,
        }
    }

    #[test]
    fn waiting_receivers_join_the_device_list_once_and_by_kind() {
        let listed = vec![ConnectDevice {
            id: Some("signed-in".into()),
            is_active: true,
            is_private_session: false,
            is_restricted: false,
            name: "Kitchen".into(),
            device_type: "Speaker".into(),
            volume_percent: Some(40),
            supports_volume: true,
        }];
        *LAN.found.lock().unwrap() = vec![
            receiver("Kitchen", Some("signed-in"), 5000, "/", false),
            receiver("Den", Some("den"), 5000, "/zc", false),
            receiver("Nest", Some(&crate::cast::device_id("Nest")), 8009, "/", true),
            receiver("Move", Some("move"), 1400, "/spotifyzc", false),
            receiver("Unresolved", None, 5000, "/", false),
        ];
        let rows = waiting_rows(&listed);
        let names: Vec<_> = rows.iter().map(|row| (row.name.as_str(), row.device_type.as_str())).collect();
        assert_eq!(names, [("Den", "Speaker"), ("Nest", "CastAudio"), ("Move", "Speaker")]);
        assert!(rows.iter().all(|row| !row.is_active));
        assert!(knows("den"));
        assert!(!knows("nobody"));
        let kinds: Vec<_> = super::rows(&LAN.found()).into_iter().map(|row| row.kind).collect();
        assert_eq!(kinds, ["connect", "connect", "cast", "sonos", "connect"]);
        LAN.clear();
        assert!(waiting_rows(&listed).is_empty());
    }
}
