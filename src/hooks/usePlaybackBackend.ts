import { useQuery } from "@tanstack/react-query";
import { getPlaybackBackend } from "../api/playback";

/**
 * Which backend is supplying audio right now.
 *
 * Free Spotify accounts can't stream through librespot, so their audio comes
 * from YouTube Music while Spotify keeps supplying metadata, artwork and
 * lyrics. The UI uses this to say so plainly rather than leaving the user to
 * wonder where the sound is coming from.
 *
 * Effectively static for a session - it only changes on re-login or when the
 * user flips the override - so it is cached hard and refetched on window focus
 * rather than polled.
 */
export function usePlaybackBackend() {
  return useQuery({
    queryKey: ["playback-backend"],
    queryFn:  getPlaybackBackend,
    staleTime: 300_000,
  });
}
