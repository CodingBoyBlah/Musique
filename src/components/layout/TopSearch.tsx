import { useEffect, useRef, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Play, X, ArrowRight } from "@/lib/icons";
import { useSearch } from "../../hooks/useSearch";
import { usePlayerStore } from "../../store/player.store";
import { playTrack, resumeOrPlay } from "../../api/playback";
import { CoverArt } from "../ui/CoverArt";
import { fmtMs } from "../../utils/fmt";
import { isMac } from "../../lib/platform";
import { EASE_OUT } from "../../lib/motion";
import { episodeToTrack } from "../../utils/episode";

/* the search field in the middle of the top bar.

   off the search page it behaves like the old Ctrl+K palette, folded into a
   dropdown under the field: quick songs / albums / artists, arrow keys to
   move, Enter to open full results. ON the search page there is no dropdown -
   the page itself is the result list, so typing just rewrites ?q= in place
   and the page follows along. */

const SECTION_HEAD: React.CSSProperties = {
  fontSize: 10.5,
  fontWeight: 700,
  letterSpacing: "0.08em",
  textTransform: "uppercase",
  color: "rgba(255, 255, 255, 0.42)",
  padding: "10px 12px 4px",
};

const ROW_TITLE: React.CSSProperties = {
  margin: 0, fontSize: 13, fontWeight: 600,
  overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
};

// small type opens its tracking back up a touch (see --type-caption-track)
const ROW_SUB: React.CSSProperties = {
  margin: "2px 0 0", fontSize: 11.5, color: "rgba(255, 255, 255, 0.5)",
  letterSpacing: "var(--type-caption-track)",
  overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
};

const KIND_TAG: React.CSSProperties = {
  fontSize: 10, fontWeight: 650, color: "rgba(255, 255, 255, 0.45)",
  letterSpacing: "0.01em",
  background: "rgba(255, 255, 255, 0.06)", padding: "2px 6px", borderRadius: 4,
};

/* No transition on the selection fill. Arrow keys move it, fast and
   repeatedly, and a fade trailing behind each keypress reads as lag - a
   keyboard action should land on the frame the key goes down. */
function rowStyle(selected: boolean): React.CSSProperties {
  return {
    display: "flex", alignItems: "center", gap: 12,
    padding: "7px 10px", borderRadius: 8, cursor: "pointer",
    background: selected ? "rgba(255, 255, 255, 0.08)" : "transparent",
  };
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
  const resultCount = (data?.tracks?.length ?? 0) + (data?.albums?.length ?? 0) + (data?.artists?.length ?? 0)
    + (data?.shows?.length ?? 0) + (data?.episodes?.length ?? 0);
  const settled = debouncedQuery === query.trim() && !isLoading && !isPlaceholderData;

  const tracks = data?.tracks?.slice(0, 4) ?? [];
  const albums = data?.albums?.slice(0, 3) ?? [];
  const artists = data?.artists?.slice(0, 2) ?? [];
  const shows = data?.shows?.slice(0, 2) ?? [];
  const episodes = data?.episodes?.slice(0, 2) ?? [];

  type FlatItem =
    | { type: "query" }
    | { type: "track"; data: typeof tracks[0] }
    | { type: "album"; data: typeof albums[0] }
    | { type: "artist"; data: typeof artists[0] }
    | { type: "show"; data: typeof shows[0] }
    | { type: "episode"; data: typeof episodes[0] };

  const items: FlatItem[] = [
    { type: "query" },
    ...tracks.map((t) => ({ type: "track" as const, data: t })),
    ...albums.map((a) => ({ type: "album" as const, data: a })),
    ...artists.map((ar) => ({ type: "artist" as const, data: ar })),
    ...shows.map((sh) => ({ type: "show" as const, data: sh })),
    ...episodes.map((ep) => ({ type: "episode" as const, data: ep })),
  ];

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
      setCurrentTrack(item.data);
      playTrack(item.data.id).catch(() => {});
    } else if (item.type === "album") {
      navigate(`/album/${item.data.id}`);
    } else if (item.type === "show") {
      navigate(`/show/${item.data.id}`);
    } else if (item.type === "episode") {
      const ep = item.data;
      const track = episodeToTrack(ep);
      setCurrentTrack(track);
      const pos = !ep.fully_played && ep.resume_position_ms ? ep.resume_position_ms : 0;
      (pos > 0 ? resumeOrPlay(track.id, pos) : playTrack(track.id)).catch(() => {});
    } else {
      navigate(`/artist/${item.data.id}`);
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
          onFocus={() => { setFocused(true); setSelectedIndex(0); }}
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
            className="glass-solid-fallback"
            initial={instantOpenRef.current ? false : { opacity: 0, y: -4, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.1, ease: EASE_OUT } }}
            transition={{ duration: 0.14, ease: EASE_OUT }}
            // keep focus in the field while picking a result
            onMouseDown={(e) => e.preventDefault()}
            style={{
              position: "absolute",
              top: "calc(100% + 8px)",
              left: "50%",
              x: "-50%",
              width: "max(100%, min(440px, 92vw))",
              transformOrigin: "top center",
              borderRadius: 12,
              background: "var(--color-popover)",
              backdropFilter: "blur(40px) saturate(1.4)",
              WebkitBackdropFilter: "blur(40px) saturate(1.4)",
              border: "1px solid var(--color-border)",
              boxShadow: "0 16px 40px rgba(0, 0, 0, 0.5)",
              overflow: "hidden",
              zIndex: 100,
            }}
          >
            <div
              ref={listRef}
              className="scroll-y"
              style={{
                maxHeight: "min(60vh, 480px)",
                overflowY: "auto",
                padding: 6,
                display: "flex",
                flexDirection: "column",
                gap: 2,
                opacity: isPlaceholderData ? 0.55 : 1,
                transition: "opacity 0.12s",
              }}
            >
              <div
                data-index="0"
                id="ts-opt-0"
                className="ts-row"
                role="option"
                aria-selected={selectedIndex === 0}
                onClick={() => handleSelect(0)}
                onMouseEnter={() => setSelectedIndex(0)}
                style={{ ...rowStyle(selectedIndex === 0), color: selectedIndex === 0 ? "#fff" : "rgba(255, 255, 255, 0.75)" }}
              >
                <div style={{
                  width: 32, height: 32, borderRadius: 7, flexShrink: 0,
                  background: selectedIndex === 0 ? "var(--color-accent)" : "rgba(255, 255, 255, 0.06)",
                  display: "flex", alignItems: "center", justifyContent: "center", color: "#fff",
                }}>
                  <Search size={14} strokeWidth={2.4} />
                </div>
                <div style={{ flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: 13.5, fontWeight: 600 }}>
                  Search for “{query.trim()}”
                </div>
                <ArrowRight size={14} style={{ color: "rgba(255, 255, 255, 0.3)", flexShrink: 0 }} />
              </div>

              {!settled && resultCount === 0 && (
                <div className="t-caption" style={{ padding: "10px 12px", fontSize: 12, color: "rgba(255, 255, 255, 0.5)" }}>Searching…</div>
              )}
              {settled && resultCount === 0 && (
                <div className="t-caption" style={{ padding: "10px 12px", fontSize: 12, color: "rgba(255, 255, 255, 0.5)" }}>No matches for “{query.trim()}”</div>
              )}

              {tracks.length > 0 && <div style={SECTION_HEAD}>Songs</div>}
              {tracks.map((t, i) => {
                const idx = 1 + i;
                const sel = selectedIndex === idx;
                return (
                  <div key={t.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel}
                    onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}>
                    <div style={{ position: "relative", width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                      <CoverArt url={t.album?.image_url} alt={t.name} size={34} />
                      {sel && (
                        <div style={{ position: "absolute", inset: 0, background: "rgba(0, 0, 0, 0.4)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                          <Play size={14} fill="#fff" strokeWidth={0} />
                        </div>
                      )}
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{t.name}</p>
                      <p style={ROW_SUB}>{t.artists.map((a) => a.name).join(", ")}</p>
                    </div>
                    <span className="tnum t-caption" style={{ fontSize: 11, color: "rgba(255, 255, 255, 0.4)" }}>
                      {fmtMs(t.duration_ms)}
                    </span>
                  </div>
                );
              })}

              {albums.length > 0 && <div style={SECTION_HEAD}>Albums</div>}
              {albums.map((a, i) => {
                const idx = 1 + tracks.length + i;
                const sel = selectedIndex === idx;
                return (
                  <div key={a.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel}
                    onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}>
                    <div style={{ width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                      <CoverArt url={a.image_url} alt={a.name} size={34} />
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{a.name}</p>
                      <p style={ROW_SUB}>{a.artists.map((ar) => ar.name).join(", ")}</p>
                    </div>
                    <span style={KIND_TAG}>Album</span>
                  </div>
                );
              })}

              {artists.length > 0 && <div style={SECTION_HEAD}>Artists</div>}
              {artists.map((ar, i) => {
                const idx = 1 + tracks.length + albums.length + i;
                const sel = selectedIndex === idx;
                return (
                  <div key={ar.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel}
                    onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}>
                    <div style={{ width: 34, height: 34, borderRadius: "50%", overflow: "hidden", flexShrink: 0 }}>
                      <CoverArt url={ar.image_url} alt={ar.name} size={34} />
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{ar.name}</p>
                    </div>
                    <span style={KIND_TAG}>Artist</span>
                  </div>
                );
              })}

              {shows.length + episodes.length > 0 && <div style={SECTION_HEAD}>Podcasts</div>}
              {shows.map((sh, i) => {
                const idx = 1 + tracks.length + albums.length + artists.length + i;
                const sel = selectedIndex === idx;
                return (
                  <div key={sh.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel}
                    onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}>
                    <div style={{ width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                      <CoverArt url={sh.image_url} alt={sh.name} size={34} />
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{sh.name}</p>
                      <p style={ROW_SUB}>{sh.publisher}</p>
                    </div>
                    <span style={KIND_TAG}>Podcast</span>
                  </div>
                );
              })}
              {episodes.map((ep, i) => {
                const idx = 1 + tracks.length + albums.length + artists.length + shows.length + i;
                const sel = selectedIndex === idx;
                return (
                  <div key={ep.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel}
                    onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}>
                    <div style={{ position: "relative", width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                      <CoverArt url={ep.image_url} alt={ep.name} size={34} />
                      {sel && (
                        <div style={{ position: "absolute", inset: 0, background: "rgba(0, 0, 0, 0.4)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                          <Play size={14} fill="#fff" strokeWidth={0} />
                        </div>
                      )}
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{ep.name}</p>
                      <p style={ROW_SUB}>{ep.show_name ?? "Episode"}</p>
                    </div>
                    <span style={KIND_TAG}>Episode</span>
                  </div>
                );
              })}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
