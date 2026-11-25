//! The app's side of the CDM host process.
//!
//! One host serves the whole app. [`Cdm::open`] starts it on first use and every handle shares
//! it; when the last handle is dropped the host is kept for [`GRACE`] (so moving from one track
//! to the next does not reload the module) and then told to exit by closing its stdin.
//!
//! Each request is one round trip under the link's lock. A watchdog kills a host that sits on a
//! request longer than its patience, which turns a hung module into an error instead of a stuck
//! decoder thread.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::link::Link;
use super::wire::{self, Reader};
use crate::errors::AppError;

/// How long an unused host is kept before it exits. Starting one and opening the module costs
/// about 300 ms, which a skip to a track that was not preloaded would otherwise pay again
/// whenever the last track's license had been let go a while before.
const GRACE: Duration = Duration::from_secs(5 * 60);

/// One CENC sample for [`Cdm::decrypt_many`].
pub struct Sample<'a> {
    /// The sample, decrypted in place.
    pub data: &'a mut [u8],
    /// 8 or 16 bytes.
    pub iv: &'a [u8],
    /// `(clear_bytes, encrypted_bytes)`; empty means the whole sample is encrypted.
    pub subsamples: &'a [(u32, u32)],
}

struct Slot {
    link: Option<Arc<Link>>,
    leases: usize,
    epoch: u64,
}

static SLOT: Mutex<Slot> = Mutex::new(Slot {
    link: None,
    leases: 0,
    epoch: 0,
});

/// Keeps the host alive while any handle holds it.
struct Lease {
    link: Arc<Link>,
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut slot = SLOT.lock().unwrap_or_else(|e| e.into_inner());
        slot.leases = slot.leases.saturating_sub(1);
        if slot.leases > 0 {
            return;
        }
        slot.epoch += 1;
        let epoch = slot.epoch;
        drop(slot);
        let _ = std::thread::Builder::new()
            .name("widevine-retire".into())
            .spawn(move || {
                std::thread::sleep(GRACE);
                let retired = {
                    let mut slot = SLOT.lock().unwrap_or_else(|e| e.into_inner());
                    if slot.leases == 0 && slot.epoch == epoch {
                        slot.link.take()
                    } else {
                        None
                    }
                };
                drop(retired); // the last Arc goes here; Link::drop stops the host
            });
    }
}

/// A handle on the CDM host. Cheap to clone; every method blocks and is safe from any thread.
///
/// A challenge and the license that answers it are matched by session: [`Cdm::accept`] feeds
/// the session the same handle's last [`Cdm::challenge`] opened. Clones start with no pending
/// session, so give each concurrent exchange its own clone, or use [`Cdm::exchange`].
pub struct Cdm {
    lease: Arc<Lease>,
    pending: Mutex<Option<Vec<u8>>>,
}

impl Clone for Cdm {
    fn clone(&self) -> Self {
        Self {
            lease: self.lease.clone(),
            pending: Mutex::new(None),
        }
    }
}

impl Cdm {
    /// Opens a handle, starting the host if none is running.
    pub fn open() -> Result<Cdm, AppError> {
        let module = super::module_path()
            .ok_or_else(|| AppError::Playback("the widevine module is not installed".into()))?;
        let mut slot = SLOT.lock().unwrap_or_else(|e| e.into_inner());
        let link = match &slot.link {
            Some(link) if !link.is_dead() => link.clone(),
            _ => {
                slot.link = None;
                let link = Link::start(&module)?;
                slot.link = Some(link.clone());
                link
            }
        };
        slot.leases += 1;
        slot.epoch += 1; // cancels a pending retirement
        Ok(Cdm {
            lease: Arc::new(Lease { link }),
            pending: Mutex::new(None),
        })
    }

    /// The module's own version string, as it reports it.
    pub fn module_version(&self) -> &str {
        &self.lease.link.module_version
    }

    /// The CDM interface version the module was opened with (10 or 11).
    pub fn interface_version(&self) -> u32 {
        self.lease.link.interface
    }

    /// Opens a temporary session for `init_data` (a `pssh` box, see [`super::pssh`]) and
    /// answers the license request to send to the license server.
    pub fn challenge(&self, init_data: &[u8]) -> Result<Vec<u8>, AppError> {
        let (session, request) = self.lease.link.call_into(wire::GENERATE, &[init_data], |answer| {
            let mut r = Reader::new(answer);
            let session = r
                .u16()
                .and_then(|len| r.take(len as usize))
                .map_err(|_| AppError::Playback("the widevine host answered garbage".into()))?
                .to_vec();
            Ok((session, r.rest().to_vec()))
        })?;
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(session);
        Ok(request)
    }

    /// Feeds the license server's response to the session the last challenge opened.
    pub fn accept(&self, license: &[u8]) -> Result<(), AppError> {
        let session = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .unwrap_or_default();
        let len = (session.len() as u16).to_le_bytes();
        let answer = self.lease.link.call(wire::UPDATE, &[&len, &session, license])?;
        let usable = Reader::new(&answer).u32().unwrap_or(0);
        log::debug!("widevine: license accepted, {usable} usable key(s)");
        Ok(())
    }

    /// One whole license exchange, serialized against every other `exchange` on this host:
    /// makes the challenge, hands it to `fetch` (which asks the license server), and feeds the
    /// answer back.
    pub fn exchange(
        &self,
        init_data: &[u8],
        fetch: impl FnOnce(Vec<u8>) -> Result<Vec<u8>, AppError>,
    ) -> Result<(), AppError> {
        let _turn = self
            .lease
            .link
            .licensing
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let request = self.challenge(init_data)?;
        let license = fetch(request)?;
        self.accept(&license)
    }

    /// Decrypts one CENC (`cenc`, AES-CTR) sample in place. `subsamples` are
    /// `(clear_bytes, encrypted_bytes)` pairs; empty means the whole sample is encrypted.
    /// `iv` is 8 or 16 bytes.
    pub fn decrypt(
        &self,
        data: &mut [u8],
        key_id: &[u8],
        iv: &[u8],
        subsamples: &[(u32, u32)],
    ) -> Result<(), AppError> {
        let iv = iv16(iv)?;
        let kid = key_id_len(key_id)?;
        // One request buffer: the sample is copied in, so `data` is free to take the answer.
        let mut body = Vec::with_capacity(21 + key_id.len() + subsamples.len() * 8 + data.len());
        body.push(kid);
        body.extend_from_slice(key_id);
        body.extend_from_slice(&iv);
        body.extend_from_slice(&(subsamples.len() as u32).to_le_bytes());
        for &(clear, cipher) in subsamples {
            body.extend_from_slice(&clear.to_le_bytes());
            body.extend_from_slice(&cipher.to_le_bytes());
        }
        body.extend_from_slice(data);
        self.lease.link.call_into(
            wire::DECRYPT,
            &[&body],
            |answer| {
                if answer.len() != data.len() {
                    return Err(AppError::Playback(
                        "the widevine host answered a sample of the wrong size".into(),
                    ));
                }
                data.copy_from_slice(answer);
                Ok(())
            },
        )
    }

    /// Decrypts several samples that share one key in a single round trip.
    pub fn decrypt_many(&self, key_id: &[u8], samples: &mut [Sample<'_>]) -> Result<(), AppError> {
        if samples.is_empty() {
            return Ok(());
        }
        let kid = key_id_len(key_id)?;
        let total: usize = samples.iter().map(|s| s.data.len() + 24 + s.subsamples.len() * 8).sum();
        let mut body = Vec::with_capacity(5 + key_id.len() + total);
        body.push(kid);
        body.extend_from_slice(key_id);
        body.extend_from_slice(&(samples.len() as u32).to_le_bytes());
        for s in samples.iter() {
            body.extend_from_slice(&iv16(s.iv)?);
            body.extend_from_slice(&(s.subsamples.len() as u32).to_le_bytes());
            for &(clear, cipher) in s.subsamples {
                body.extend_from_slice(&clear.to_le_bytes());
                body.extend_from_slice(&cipher.to_le_bytes());
            }
            body.extend_from_slice(&(s.data.len() as u32).to_le_bytes());
            body.extend_from_slice(s.data);
        }
        self.lease.link.call_into(wire::DECRYPT_MANY, &[&body], |answer| {
            let want: usize = samples.iter().map(|s| s.data.len()).sum();
            if answer.len() != want {
                return Err(AppError::Playback(
                    "the widevine host answered samples of the wrong size".into(),
                ));
            }
            let mut at = 0;
            for s in samples.iter_mut() {
                let n = s.data.len();
                s.data.copy_from_slice(&answer[at..at + n]);
                at += n;
            }
            Ok(())
        })
    }
}

/// Stops the host now, whoever holds it. Handles still alive fail their next call. Used before
/// removing the module from disk (Windows will not delete a loaded library).
pub fn shutdown() {
    let link = {
        let mut slot = SLOT.lock().unwrap_or_else(|e| e.into_inner());
        slot.epoch += 1;
        slot.link.take()
    };
    if let Some(link) = link {
        link.stop();
    }
}

fn iv16(iv: &[u8]) -> Result<[u8; 16], AppError> {
    let mut out = [0u8; 16];
    match iv.len() {
        8 | 16 => {
            out[..iv.len()].copy_from_slice(iv);
            Ok(out)
        }
        n => Err(AppError::Playback(format!("a CENC iv is 8 or 16 bytes, not {n}"))),
    }
}

fn key_id_len(key_id: &[u8]) -> Result<u8, AppError> {
    u8::try_from(key_id.len())
        .ok()
        .filter(|&n| n > 0)
        .ok_or_else(|| AppError::Playback("bad key id".into()))
}

