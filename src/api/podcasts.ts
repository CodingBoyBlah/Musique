import { invoke } from "@tauri-apps/api/core";
import type { EpisodeItem, EpisodePage, ShowDetail, ShowItem } from "../types/podcast";

export const getShow = (id: string): Promise<ShowDetail> => invoke("get_show", { id });

export const getShowEpisodes = (id: string, offset: number): Promise<EpisodePage> =>
  invoke("get_show_episodes", { id, offset });

export const getEpisode = (id: string): Promise<EpisodeItem> => invoke("get_episode", { id });

export const getSavedShows = (): Promise<ShowItem[]> => invoke("get_saved_shows");

export const saveShow = (id: string): Promise<void> => invoke("save_show", { id });

export const unsaveShow = (id: string): Promise<void> => invoke("unsave_show", { id });

export const isShowSaved = (id: string): Promise<boolean> => invoke("is_show_saved", { id });
