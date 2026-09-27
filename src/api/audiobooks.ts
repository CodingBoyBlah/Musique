import { invoke } from "@tauri-apps/api/core";
import type { AudiobookDetail, AudiobookItem, EpisodeItem } from "../types/podcast";

export const getAudiobook = (id: string): Promise<AudiobookDetail> => invoke("get_audiobook", { id });

export const getAudiobookChapters = (
  id: string,
  offset: number,
): Promise<{ chapters: EpisodeItem[]; next_offset: number | null }> =>
  invoke("get_audiobook_chapters", { id, offset });

export const getSavedAudiobooks = (): Promise<AudiobookItem[]> => invoke("get_saved_audiobooks");

export const saveAudiobook = (id: string): Promise<void> => invoke("save_audiobook", { id });

export const unsaveAudiobook = (id: string): Promise<void> => invoke("unsave_audiobook", { id });

export const isAudiobookSaved = (id: string): Promise<boolean> => invoke("is_audiobook_saved", { id });
