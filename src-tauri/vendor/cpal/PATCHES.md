# Local patch

Source: CPAL 0.16.0 from crates.io, licensed under Apache-2.0.

In `src/host/coreaudio/macos/property_listener.rs`, retain `Send` on the
boxed property callback and require it in the listener constructor. Both
existing callers already supply callbacks with thread-safe captures. This
lets CoreAudio output streams satisfy the `Send` bounds required by the
librespot sink and Tauri's managed state without changing playback behavior.

All other copied upstream files are unchanged.
