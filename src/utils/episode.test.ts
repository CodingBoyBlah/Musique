import { describe, it, expect } from "vitest";
import { episodeLength, episodeProgress, episodeToTrack, isEpisodeId } from "./episode";
import type { EpisodeItem } from "../types/podcast";

const ep: EpisodeItem = {
  id: "abc",
  name: "Pilot",
  description: null,
  image_url: "https://img",
  release_date: "2024-01-02",
  duration_ms: 3_900_000,
  explicit: false,
  is_playable: true,
  resume_position_ms: 1_800_000,
  fully_played: false,
  show_id: "show1",
  show_name: "The Show",
  publisher: "Pub",
};

describe("episode helpers", () => {
  it("round-trips the episode uri", () => {
    const t = episodeToTrack(ep);
    expect(t.id).toBe("spotify:episode:abc");
    expect(isEpisodeId(t.id)).toBe(true);
    expect(isEpisodeId("4uLU6hMCjMI75M1A2tKUQC")).toBe(false);
    expect(t.album?.name).toBe("The Show");
  });

  it("formats lengths", () => {
    expect(episodeLength(42 * 60000)).toBe("42 min");
    expect(episodeLength(65 * 60000)).toBe("1 hr 5 min");
    expect(episodeLength(120 * 60000)).toBe("2 hr");
  });

  it("reports progress", () => {
    expect(episodeProgress(ep)?.label).toBe("35 min left");
    expect(episodeProgress({ ...ep, fully_played: true })?.label).toBe("Played");
    expect(episodeProgress({ ...ep, resume_position_ms: 0 })).toBeNull();
  });
});
