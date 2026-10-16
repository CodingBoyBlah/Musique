# Spotify playback and listening history

The Spotify backend already streams Spotify audio through librespot and requires
Premium. The YouTube backend plays matched YouTube audio. Playing a Spotify track's
metadata through YouTube does not make it a Spotify listen.

## Using an official Spotify player

For Spotify's own client to own playback and listening telemetry, open Spotify on
your computer or phone and choose it in Musique's device picker. Selecting another
song in Musique now keeps that device selected. Pause, resume, seek and skip use the
Spotify remote controls. The official player must remain available.

Spotify documents this remote control model in its
[Connect help](https://support.spotify.com/in-en/article/spotify-connect/) and
[Start/Resume Playback API](https://developer.spotify.com/documentation/web-api/reference/start-a-users-playback).
The API requires Premium. This uses an official player to produce its own listening
events, but Musique still cannot inspect or guarantee Spotify's statistics ingestion.

## Local Spotify playback

Musique registers a Connect device and separately submits `RawCoreStream` listening
events. Connect's now-playing state alone does not establish that a listen reached
history or Sound Capsule. Reports carry the track, audio identity, context and
listening time. Listening time now comes from audio pulled by the output, excluding
paused silence, seek jumps and buffered audio discarded by a skip. The silent
fallback and YouTube playback produce no Spotify listening reports.

The event service is undocumented. The upstream
[librespot history-reporting PR](https://github.com/librespot-org/librespot/pull/1759)
reports a successful recently-played check on one account, explicitly does not
establish Wrapped or royalty accounting, and was closed with `wontfix`.
An accepted submission is not evidence that Sound Capsule counted it. Reports are
held in memory and can be lost on a crash or prolonged network failure.

## Checking Sound Capsule

[Spotify's Sound Capsule help](https://support.spotify.com/in-en/article/your-sound-capsule/)
lists Premium on mobile as the requirement, excluding Premium Mini. It does not
document third-party event ingestion or an API to verify it.

To verify your account, record the current history and Sound Capsule totals, listen
normally, then check the official app after its statistics update. Test local
Musique playback and playback on an official device separately. A
`[listening] submitted` log only confirms that the event endpoint accepted a local
report. The official app or an extended streaming-history export is the evidence
that Spotify recorded it. No Sound Capsule ingestion check was performed as part
of this code change.
