import { useEffect, useMemo, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useQueryClient } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Play, X, ChevronRight } from "@/lib/icons";
import { useSearch } from "../../hooks/useSearch";
import { cancelHoverPrefetch, prefetchAlbum, prefetchArtist, prefetchPlaylist, prefetchShow, preloadRoute, scheduleHoverPrefetch } from "../../lib/prefetch";
import { usePlayerStore } from "../../store/player.store";
import { playTrack, resumeOrPlay } from "../../api/playback";
import { CoverArt } from "../ui/CoverArt";
import { fmtMs } from "../../utils/fmt";
import { isMac } from "../../lib/platform";
import { EASE_OUT } from "../../lib/motion";
import { rankSearch, type RankedResult } from "../../lib/searchRank";
import { episodeToTrack } from "../../utils/episode";

/* the search field in the middle of the top bar.

   off the search page it opens a dropdown under the field: one list, best
   match first whatever its type - "radiohead" puts the artist on top,
   "creep" the song - instead of fixed Songs / Albums / Artists sections that
   made you scroll past four songs to reach the artist you typed. arrow keys
   move, Enter opens full results. ON the search page there is no dropdown -
   the page itself is the result list, so typing just rewrites ?q= in place
   and the page follows along. */

// at most this many of a type, and this many rows in all
const CAPS = { track: 4, album: 3, artist: 3, playlist: 2, show: 2, episode: 2 } as const;
const MAX_ROWS = 8;

function joinNames(list: { name: string }[] | undefined): string {
  return (list ?? []).map((a) => a.name).join(", ");
}

function albumKind(t: string | undefined): string {
  if (t === "single") return "Single";
  if (t === "compilation") return "Compilation";
  return "Album";
}

type RowMeta = { image: string | null; title: string; sub: string; round: boolean; playable: boolean };

// what a row says under its title: the kind of thing it is, then who made it
function rowMeta(r: RankedResult): RowMeta {
  switch (r.type) {
    case "artist":
      return { image: r.item.image_url, title: r.item.name, sub: "Artist", round: true, playable: false };
    case "track":
      return { image: r.item.album?.image_url ?? null, title: r.item.name, sub: `Song · ${joinNames(r.item.artists)}`, round: false, playable: true };
    case "album":
      return { image: r.item.image_url, title: r.item.name, sub: `${albumKind(r.item.album_type)} · ${joinNames(r.item.artists)}`, round: false, playable: false };
    case "playlist":
      return { image: r.item.image_url, title: r.item.name, sub: r.item.owner_name ? `Playlist · ${r.item.owner_name}` : "Playlist", round: false, playable: false };
    case "show":
      return { image: r.item.image_url, title: r.item.name, sub: `Podcast · ${r.item.publisher}`, round: false, playable: false };
    case "episode":
      return { image: r.item.image_url, title: r.item.name, sub: r.item.show_name ? `Episode · ${r.item.show_name}` : "Episode", round: false, playable: true };
  }
}

export function TopSearch() {
  const navigate = useNavigate();
  const location = useLocation();
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const onSearchPage = location.pathname === "/search";
  const urlQuery = onSearchPage ? (new URLSearchParams(location.search).get("q") ?? "") : "";

  const [query, setQuery] = useState(urlQuery);
  const [debouncedQuery, setDebouncedQuery] = useState(urlQuery.trim());
  const [focused, setFocused] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  /* the last ?q= this field wrote itself. when the url changes to that value
     it is our own echo and must not overwrite what is still being typed. */
  const pushedRef = useRef<string | null>(null);
  const qc = useQueryClient();
  /* Ctrl+K is a shortcut people hit many times a day: when it brings back a
     dropdown that already has a query in it, the results appear at once, with
     no entrance. Only a dropdown opened by typing gets the short fade. */
  const instantOpenRef = useRef(false);

  // the field mirrors the url: full query on the search page, empty elsewhere.
  // keyed on the navigation itself, so opening a result from the dropdown
  // (home -> album, both "no query") still clears what was typed
  useEffect(() => {
    if (onSearchPage && urlQuery === pushedRef.current) return;
    pushedRef.current = null;
    setQuery(urlQuery);
    setDebouncedQuery(urlQuery.trim());
  }, [location.key, onSearchPage, urlQuery]);

  useEffect(() => {
    // short: every millisecond here sits between a keystroke and the page
    // responding to it. keepPreviousData dims the old results meanwhile, so a
    // shorter wait costs no flashing.
    const t = setTimeout(() => setDebouncedQuery(query.trim()), onSearchPage ? 120 : 110);
    return () => clearTimeout(t);
  }, [query, onSearchPage]);

  // on the search page, typing drives the page. a blank field is left alone:
  // the page sends an empty query home, which is not what clearing to retype means
  useEffect(() => {
    if (!onSearchPage || !debouncedQuery || debouncedQuery === urlQuery.trim()) return;
    pushedRef.current = debouncedQuery;
    navigate(`/search?q=${encodeURIComponent(debouncedQuery)}`, { replace: true });
  }, [debouncedQuery, onSearchPage, urlQuery, navigate]);

  // Ctrl/Cmd+K jumps to the field from anywhere
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        instantOpenRef.current = true;
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const hasQuery = query.trim().length > 0;
  const open = focused && hasQuery && !onSearchPage;
  const { data, isLoading, isPlaceholderData } = useSearch(onSearchPage ? "" : debouncedQuery);
  const settled = debouncedQuery === query.trim() && !isLoading && !isPlaceholderData;

  // rows re-render on every arrow key; the merge only needs to rerun on new results
  const ranked = useMemo(() => rankSearch(data, debouncedQuery, CAPS, MAX_ROWS), [data, debouncedQuery]);
  type FlatItem = { type: "query" } | RankedResult;
  const items: FlatItem[] = [{ type: "query" }, ...ranked];

  function prefetchFor(r: RankedResult) {
    if (r.type === "album") prefetchAlbum(qc, r.item.id);
    else if (r.type === "artist") prefetchArtist(qc, r.item.id);
    else if (r.type === "playlist") prefetchPlaylist(qc, r.item.id);
    else if (r.type === "show") prefetchShow(qc, r.item.id);
  }

  function openResults() {
    const q = query.trim();
    if (!q) return;
    pushedRef.current = q;
    navigate(`/search?q=${encodeURIComponent(q)}`, { replace: onSearchPage });
    inputRef.current?.blur();
  }

  function handleSelect(index: number) {
    const item = items[index];
    if (!item) return;
    if (item.type === "query") {
      openResults();
      return;
    }
    if (item.type === "track") {
      setCurrentTrack(item.item);
      playTrack(item.item.id).catch(() => {});
    } else if (item.type === "album") {
      navigate(`/album/${item.item.id}`);
    } else if (item.type === "playlist") {
      navigate(`/playlist/${item.item.id}`);
    } else if (item.type === "show") {
      navigate(`/show/${item.item.id}`);
    } else if (item.type === "episode") {
      const ep = item.item;
      const track = episodeToTrack(ep);
      setCurrentTrack(track);
      const pos = !ep.fully_played && ep.resume_position_ms ? ep.resume_position_ms : 0;
      (pos > 0 ? resumeOrPlay(track.id, pos) : playTrack(track.id)).catch(() => {});
    } else {
      navigate(`/artist/${item.item.id}`);
    }
    inputRef.current?.blur();
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Escape") {
      e.preventDefault();
      inputRef.current?.blur();
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (open) handleSelect(selectedIndex);
      else openResults();
    } else if (open && e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => (i + 1) % items.length);
    } else if (open && e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => (i - 1 + items.length) % items.length);
    }
  }

  useEffect(() => {
    const el = listRef.current?.querySelector(`[data-index="${selectedIndex}"]`) as HTMLElement | null;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  return (
    <div style={{ position: "relative", width: "100%" }}>
      <div className="tb-search" data-focused={focused ? "true" : undefined} onMouseDown={(e) => {
        // clicks on the pill's padding still land in the field
        if (e.target !== inputRef.current) { e.preventDefault(); inputRef.current?.focus(); }
      }}>
        <Search size={15} strokeWidth={2.1} style={{ flexShrink: 0, color: focused ? "#fff" : "var(--color-text-dim)" }} />
        <input
          ref={inputRef}
          value={query}
          // a synthesised Ctrl+K can land its \x0B in the field on WebView2;
          // an invisible control char would still count as "has text"
          onChange={(e) => { setQuery(e.target.value.replace(/[\x00-\x1f\x7f]/g, "")); setSelectedIndex(0); }}
          onFocus={() => { setFocused(true); setSelectedIndex(0); preloadRoute("search"); }}
          onBlur={() => { setFocused(false); instantOpenRef.current = false; }}
          onKeyDown={handleKeyDown}
          placeholder="What do you want to play?"
          spellCheck={false}
          aria-label="Search"
          aria-expanded={open}
          aria-controls="top-search-results"
          aria-activedescendant={open ? `ts-opt-${selectedIndex}` : undefined}
          aria-autocomplete="list"
          role="combobox"
        />
        {query ? (
          <button
            className="tb-search-clear"
            aria-label="Clear search"
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => { setQuery(""); inputRef.current?.focus(); }}
          >
            <X size={13} strokeWidth={2.2} />
          </button>
        ) : !focused && (
          <kbd className="tb-search-kbd">{isMac ? "⌘K" : "Ctrl K"}</kbd>
        )}
      </div>

      <AnimatePresence>
        {open && (
          <motion.div
            id="top-search-results"
            role="listbox"
            aria-label="Search suggestions"
            className="ts-pop glass-solid-fallback"
            // grows out of the field it belongs to, and goes back into it
            initial={instantOpenRef.current ? false : { opacity: 0, y: -6, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.12, ease: EASE_OUT } }}
            transition={{ type: "spring", bounce: 0, duration: 0.28 }}
            // keep focus in the field while picking a result
            onMouseDown={(e) => e.preventDefault()}
            style={{ x: "-50%" }}
          >
            <div ref={listRef} className="ts-list scroll-y" data-stale={isPlaceholderData ? "true" : undefined}>
              <div
                data-index="0"
                id="ts-opt-0"
                className="ts-row ts-row--query"
                role="option"
                aria-selected={selectedIndex === 0}
                data-active={selectedIndex === 0 ? "true" : undefined}
                onClick={() => handleSelect(0)}
                onMouseEnter={() => setSelectedIndex(0)}
              >
                <div className="ts-art ts-art--icon">
                  <Search size={16} strokeWidth={2.2} />
                </div>
                <div className="ts-text">
                  <p className="ts-title">{query.trim()}</p>
                  <p className="ts-sub">See all results</p>
                </div>
                <kbd className="ts-kbd ts-reveal" aria-hidden>↵</kbd>
              </div>

              {(ranked.length > 0 || !settled) && <div className="ts-sep" role="presentation" />}

              {!settled && ranked.length === 0 && [0, 1, 2].map((i) => (
                <div key={i} className="ts-row ts-row--ghost" aria-hidden>
                  <div className="ts-art" />
                  <div className="ts-text">
                    <span className="ts-bar" style={{ width: `${62 - i * 14}%` }} />
                    <span className="ts-bar ts-bar--sub" style={{ width: `${38 - i * 6}%` }} />
                  </div>
                </div>
              ))}
              {settled && ranked.length === 0 && (
                <p className="ts-empty">No matches for “{query.trim()}”</p>
              )}

              {ranked.map((r, i) => {
                const idx = 1 + i;
                const sel = selectedIndex === idx;
                const m = rowMeta(r);
                const prefetchable = r.type !== "track" && r.type !== "episode";
                return (
                  <div
                    key={`${r.type}:${r.item.id}`}
                    data-index={idx}
                    id={`ts-opt-${idx}`}
                    className="ts-row"
                    role="option"
                    aria-selected={sel}
                    aria-label={`${m.title}, ${m.sub}`}
                    data-active={sel ? "true" : undefined}
                    onClick={() => handleSelect(idx)}
                    onMouseEnter={() => {
                      setSelectedIndex(idx);
                      if (prefetchable) scheduleHoverPrefetch(() => prefetchFor(r), 50);
                    }}
                    onMouseLeave={prefetchable ? cancelHoverPrefetch : undefined}
                    onPointerDown={prefetchable ? () => { cancelHoverPrefetch(); prefetchFor(r); } : undefined}
                  >
                    <div className={m.round ? "ts-art ts-art--round" : "ts-art"}>
                      <CoverArt url={m.image} alt="" size={40} />
                      {m.playable && sel && (
                        <div className="ts-art-play"><Play size={14} fill="#fff" strokeWidth={0} /></div>
                      )}
                    </div>
                    <div className="ts-text">
                      <p className="ts-title">{m.title}</p>
                      <p className="ts-sub">{m.sub}</p>
                    </div>
                    {r.type === "track" ? (
                      <span className="ts-trail tnum">{fmtMs(r.item.duration_ms)}</span>
                    ) : (
                      <ChevronRight size={15} strokeWidth={2.2} className="ts-reveal ts-chev" aria-hidden />
                    )}
                  </div>
                );
              })}
            </div>

            <div className="ts-foot" aria-hidden>
              <span><kbd className="ts-kbd">↑</kbd><kbd className="ts-kbd">↓</kbd>Navigate</span>
              <span><kbd className="ts-kbd">↵</kbd>Open</span>
              <span><kbd className="ts-kbd">esc</kbd>Close</span>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
