import { invoke } from "@tauri-apps/api/core";
import type { EpisodeItem, EpisodeMedia, EpisodePage, ShowDetail, ShowItem, Transcript } from "../types/podcast";

export type { Transcript, EpisodeMedia };

export const getShow = (id: string): Promise<ShowDetail> => invoke("get_show", { id });

export const getShowEpisodes = (id: string, offset: number): Promise<EpisodePage> =>
  invoke("get_show_episodes", { id, offset });

export const getEpisode = (id: string): Promise<EpisodeItem> => invoke("get_episode", { id });

export const getSavedShows = (): Promise<ShowItem[]> => invoke("get_saved_shows");

export const saveShow = (id: string): Promise<void> => invoke("save_show", { id });

export const unsaveShow = (id: string): Promise<void> => invoke("unsave_show", { id });

export const isShowSaved = (id: string): Promise<boolean> => invoke("is_show_saved", { id });

// spotify's synced read-along transcript, null when the episode has none
export const getEpisodeTranscript = (id: string): Promise<Transcript | null> =>
  invoke("get_episode_transcript", { id });

// video podcast info: whether it's video, and a preview clip to loop
export const getEpisodeMedia = (id: string): Promise<EpisodeMedia> => invoke("get_episode_media", { id });
