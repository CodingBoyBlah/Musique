use std::{collections::HashMap, io::Write, time::Duration};

use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use bytes::Bytes;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::{Error, FileId, SpotifyId, packet::PacketType, util::SeqGenerator};

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct AudioKey(pub [u8; 16]);

#[derive(Debug, Error)]
pub enum AudioKeyError {
    #[error("audio key error")]
    AesKey,
    #[error("other end of channel disconnected")]
    Channel,
    #[error("unexpected packet type {0}")]
    Packet(u8),
    #[error("sequence {0} not pending")]
    Sequence(u32),
    #[error("audio key response timeout")]
    Timeout,
}

impl From<AudioKeyError> for Error {
    fn from(err: AudioKeyError) -> Self {
        match err {
            AudioKeyError::AesKey => Error::unavailable(err),
            AudioKeyError::Channel => Error::aborted(err),
            AudioKeyError::Sequence(_) => Error::aborted(err),
            AudioKeyError::Packet(_) => Error::unimplemented(err),
            AudioKeyError::Timeout => Error::aborted(err),
        }
    }
}

component! {
    AudioKeyManager : AudioKeyManagerInner {
        sequence: SeqGenerator<u32> = SeqGenerator::new(0),
        pending: HashMap<u32, oneshot::Sender<Result<AudioKey, Error>>> = HashMap::new(),
    }
}

impl AudioKeyManager {
    pub(crate) fn dispatch(&self, cmd: PacketType, mut data: Bytes) -> Result<(), Error> {
        let seq = BigEndian::read_u32(data.split_to(4).as_ref());

        let sender = self
            .lock(|inner| inner.pending.remove(&seq))
            .ok_or(AudioKeyError::Sequence(seq))?;

        match cmd {
            PacketType::AesKey => {
                let mut key = [0u8; 16];
                key.copy_from_slice(data.as_ref());
                sender
                    .send(Ok(AudioKey(key)))
                    .map_err(|_| AudioKeyError::Channel)?
            }
            PacketType::AesKeyError => {
                error!(
                    "error audio key {:x} {:x}",
                    data.as_ref()[0],
                    data.as_ref()[1]
                );
                sender
                    .send(Err(AudioKeyError::AesKey.into()))
                    .map_err(|_| AudioKeyError::Channel)?
            }
            _ => {
                trace!("Did not expect {cmd:?} AES key packet with data {data:#?}");
                return Err(AudioKeyError::Packet(cmd as u8).into());
            }
        }

        Ok(())
    }

    pub async fn request(&self, track: SpotifyId, file: FileId) -> Result<AudioKey, Error> {
        // PATCHED (spotify-client vendor): the AES audio-key fetch over the AP
        // occasionally gets no response in time ("Audio key response timeout"),
        // which made librespot continue without the key → garbage decode → the
        // track was skipped as unplayable ("Can't play this track"). Upstream
        // issues ONE request with a single 1.5s timeout and no retry.
        //
        // We register ONE pending entry (one sequence) and keep it alive for the
        // whole wait, "nudging" the AP by RESENDING THE SAME sequence if it's
        // quiet. This fixes both failure modes without the race that a
        // fresh-sequence-per-retry has (where a response arriving just after a
        // short window is dropped because the next request already moved on):
        //   • slow AP  → we simply keep waiting; a late response still resolves.
        //   • dropped packet → a nudge re-asks; any matching response resolves.
        // A genuinely unanswered key (restricted/wedged session) still fails
        // after the budget — the app then rebuilds a fresh session and retries
        // (see commands::playback::retry_play_track).
        const NUDGE_INTERVAL: Duration = Duration::from_millis(2000);
        const MAX_NUDGES: usize = 5; // ~10s total budget

        let (tx, mut rx) = oneshot::channel();
        let seq = self.lock(move |inner| {
            let seq = inner.sequence.get();
            inner.pending.insert(seq, tx);
            seq
        });

        if let Err(e) = self.send_key_request(seq, track, file) {
            self.lock(|inner| inner.pending.remove(&seq));
            return Err(e);
        }

        for nudge in 0..=MAX_NUDGES {
            // `rx` is Unpin, so `&mut rx` can be awaited repeatedly across the
            // timeout loop without consuming the single pending entry.
            match tokio::time::timeout(NUDGE_INTERVAL, &mut rx).await {
                Ok(Ok(Ok(key))) => return Ok(key),
                Ok(Ok(Err(e))) => return Err(e), // server AesKeyError: definitive
                Ok(Err(_recv)) => {
                    // Sender dropped by dispatch — can't recover this seq.
                    self.lock(|inner| inner.pending.remove(&seq));
                    return Err(AudioKeyError::Channel.into());
                }
                Err(_) => {
                    if nudge < MAX_NUDGES {
                        // Still pending — re-ask on the SAME seq and keep waiting.
                        let _ = self.send_key_request(seq, track, file);
                    }
                }
            }
        }

        self.lock(|inner| inner.pending.remove(&seq));
        error!("Audio key response timeout after {MAX_NUDGES} nudges");
        Err(AudioKeyError::Timeout.into())
    }

    fn send_key_request(&self, seq: u32, track: SpotifyId, file: FileId) -> Result<(), Error> {
        let mut data: Vec<u8> = Vec::new();
        data.write_all(&file.0)?;
        data.write_all(&track.to_raw())?;
        data.write_u32::<BigEndian>(seq)?;
        data.write_u16::<BigEndian>(0x0000)?;

        self.session().send_packet(PacketType::RequestKey, data)
    }
}
