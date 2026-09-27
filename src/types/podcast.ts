export interface ShowItem {
  id: string;
  name: string;
  publisher: string;
  image_url: string | null;
  description: string | null;
  total_episodes: number | null;
  explicit: boolean;
  media_type: string | null;
}

export interface EpisodeItem {
  id: string;
  name: string;
  description: string | null;
  image_url: string | null;
  release_date: string | null;
  duration_ms: number;
  explicit: boolean;
  is_playable: boolean;
  resume_position_ms: number | null;
  fully_played: boolean;
  show_id: string | null;
  show_name: string | null;
  publisher: string | null;
}

export interface ShowDetail extends ShowItem {
  html_description: string | null;
  languages: string[];
  episodes: EpisodeItem[];
  next_offset: number | null;
}

export interface EpisodePage {
  episodes: EpisodeItem[];
  next_offset: number | null;
}

