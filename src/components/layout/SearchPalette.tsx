import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Play, X, ArrowRight } from "@/lib/icons";
import { useUIStore } from "../../store/ui.store";
import { useSearch } from "../../hooks/useSearch";
import { usePlayerStore } from "../../store/player.store";
import { playTrack } from "../../api/playback";
import { CoverArt } from "../ui/CoverArt";
import { fmtMs } from "../../utils/fmt";

export function SearchPalette() {
  const isOpen = useUIStore((s) => s.searchPaletteOpen);
  const setIsOpen = useUIStore((s) => s.setSearchPaletteOpen);
  const setCurrentTrack = usePlayerStore((s) => s.setCurrentTrack);
  const navigate = useNavigate();

  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(query.trim());
    }, 110);
    return () => clearTimeout(timer);
  }, [query]);

  const { data, isLoading, isPlaceholderData } = useSearch(debouncedQuery);
  const resultCount = (data?.tracks?.length ?? 0) + (data?.albums?.length ?? 0) + (data?.artists?.length ?? 0);
  const hasQuery = query.trim().length > 0;

  // Focus input as soon as the palette mounts (no artificial delay)
  useEffect(() => {
    if (isOpen) {
      setSelectedIndex(0);
      requestAnimationFrame(() => inputRef.current?.focus());
    } else {
      setQuery("");
      setDebouncedQuery("");
    }
  }, [isOpen]);

  // Build flattened items list for keyboard navigation
  const tracks = data?.tracks?.slice(0, 4) ?? [];
  const albums = data?.albums?.slice(0, 3) ?? [];
  const artists = data?.artists?.slice(0, 2) ?? [];

  type FlatItem =
    | { type: "query"; label: string }
    | { type: "track"; data: typeof tracks[0] }
    | { type: "album"; data: typeof albums[0] }
    | { type: "artist"; data: typeof artists[0] };

  const items: FlatItem[] = [
    { type: "query", label: query.trim() || "Search in Musique" },
    ...tracks.map((t) => ({ type: "track" as const, data: t })),
    ...albums.map((a) => ({ type: "album" as const, data: a })),
    ...artists.map((ar) => ({ type: "artist" as const, data: ar })),
  ];

  function handleSelect(index: number) {
    const item = items[index];
    if (!item) return;

    if (item.type === "query") {
      if (query.trim()) {
        navigate(`/search?q=${encodeURIComponent(query.trim())}`);
      } else {
        navigate("/search");
      }
      setIsOpen(false);
    } else if (item.type === "track") {
      setCurrentTrack(item.data);
      playTrack(item.data.id).catch(() => {});
      setIsOpen(false);
    } else if (item.type === "album") {
      navigate(`/album/${item.data.id}`);
      setIsOpen(false);
    } else if (item.type === "artist") {
      navigate(`/artist/${item.data.id}`);
      setIsOpen(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      setIsOpen(false);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % Math.max(1, items.length));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + items.length) % Math.max(1, items.length));
    } else if (e.key === "Enter") {
      e.preventDefault();
      handleSelect(selectedIndex);
    }
  }

  // Scroll active item into view
  useEffect(() => {
    if (!listRef.current) return;
    const el = listRef.current.querySelector(`[data-index="${selectedIndex}"]`) as HTMLElement | null;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  return (
    <AnimatePresence>
      {isOpen && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            zIndex: 1000,
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "center",
            paddingTop: "14vh",
          }}
        >
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.18 }}
            onClick={() => setIsOpen(false)}
            style={{
              position: "absolute",
              inset: 0,
              background: "rgba(0, 0, 0, 0.65)",
              backdropFilter: "blur(10px)",
              WebkitBackdropFilter: "blur(10px)",
            }}
          />

          {/* Arc Command Palette */}
          <motion.div
            initial={{ opacity: 0, scale: 0.97, y: -8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: -6 }}
            transition={{ type: "spring", stiffness: 520, damping: 36, mass: 0.7 }}
            style={{
              position: "relative",
              width: "min(620px, 92vw)",
              display: "flex",
              flexDirection: "column",
              borderRadius: 16,
              background: "rgba(22, 22, 26, 0.96)",
              backdropFilter: "blur(32px) saturate(140%)",
              WebkitBackdropFilter: "blur(32px) saturate(140%)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              boxShadow: "0 24px 72px rgba(0, 0, 0, 0.7), inset 0 1px 0 rgba(255, 255, 255, 0.08)",
              overflow: "hidden",
              pointerEvents: "auto",
            }}
            onKeyDown={handleKeyDown}
          >
            {/* Input Header */}
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "16px 18px",
                borderBottom: hasQuery ? "1px solid rgba(255, 255, 255, 0.08)" : "none",
              }}
            >
              <Search size={20} strokeWidth={2.2} style={{ color: "var(--color-accent)", flexShrink: 0 }} />
              <input
                ref={inputRef}
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setSelectedIndex(0);
                }}
                placeholder="Search songs, albums, artists..."
                style={{
                  flex: 1,
                  background: "transparent",
                  border: "none",
                  outline: "none",
                  color: "#ffffff",
                  fontSize: 16,
                  fontWeight: 500,
                  fontFamily: "inherit",
                }}
              />
              {query && (
                <button
                  onClick={() => {
                    setQuery("");
                    inputRef.current?.focus();
                  }}
                  style={{
                    background: "transparent",
                    border: "none",
                    color: "rgba(255, 255, 255, 0.4)",
                    cursor: "pointer",
                    padding: 2,
                    display: "flex",
                  }}
                >
                  <X size={15} />
                </button>
              )}
              <span
                style={{
                  fontSize: 10.5,
                  fontWeight: 650,
                  letterSpacing: "0.04em",
                  color: "rgba(255, 255, 255, 0.35)",
                  background: "rgba(255, 255, 255, 0.06)",
                  border: "1px solid rgba(255, 255, 255, 0.08)",
                  padding: "2px 6px",
                  borderRadius: 5,
                  userSelect: "none",
                }}
              >
                ESC
              </span>
            </div>

            {/* Results — height animates 0 → auto. With no query the palette
                collapses to just the top bar (Arc-style). */}
            <AnimatePresence initial={false}>
              {hasQuery && (
                <motion.div
                  key="results"
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: "auto", opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.16, ease: [0.25, 0.1, 0.25, 1] }}
                  style={{ overflow: "hidden" }}
                >
                  <div
                    ref={listRef}
                    className="scroll-y"
                    style={{
                      maxHeight: "min(56vh, 480px)",
                      overflowY: "auto",
                      padding: "8px",
                      display: "flex",
                      flexDirection: "column",
                      gap: 3,
                      opacity: isPlaceholderData ? 0.55 : 1,
                      transition: "opacity 0.12s",
                    }}
                  >
              {/* Primary Query Action */}
              <div
                data-index="0"
                onClick={() => handleSelect(0)}
                onMouseEnter={() => setSelectedIndex(0)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "10px 14px",
                  borderRadius: 8,
                  cursor: "pointer",
                  background: selectedIndex === 0 ? "rgba(255, 255, 255, 0.08)" : "transparent",
                  color: selectedIndex === 0 ? "#ffffff" : "rgba(255, 255, 255, 0.75)",
                  transition: "background 0.1s, color 0.1s",
                }}
              >
                <div
                  style={{
                    width: 30,
                    height: 30,
                    borderRadius: 7,
                    background: selectedIndex === 0 ? "var(--color-accent)" : "rgba(255, 255, 255, 0.06)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    color: "#ffffff",
                    flexShrink: 0,
                  }}
                >
                  <Search size={14} strokeWidth={2.4} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <span style={{ fontSize: 13.5, fontWeight: 600 }}>
                    {query.trim() ? `Search for "${query.trim()}"` : "Search in Musique"}
                  </span>
                  <span style={{ fontSize: 11.5, color: "rgba(255, 255, 255, 0.4)", marginLeft: 8 }}>
                    Full results
                  </span>
                </div>
                <ArrowRight size={14} style={{ color: "rgba(255, 255, 255, 0.3)" }} />
              </div>

              {hasQuery && (isLoading || isPlaceholderData || debouncedQuery !== query.trim()) && (
                <div style={{ padding: "12px 14px", fontSize: 12, color: "rgba(255, 255, 255, 0.4)" }}>
                  Searching Spotify…
                </div>
              )}

              {hasQuery && debouncedQuery === query.trim() && !isLoading && !isPlaceholderData && resultCount === 0 && (
                <div style={{ padding: "12px 14px", fontSize: 12, color: "rgba(255, 255, 255, 0.4)" }}>
                  No matches on Spotify
                </div>
              )}

              {/* Tracks Section */}
              {tracks.length > 0 && (
                <>
                  <div
                    style={{
                      fontSize: 10.5,
                      fontWeight: 700,
                      letterSpacing: "0.08em",
                      textTransform: "uppercase",
                      color: "rgba(255, 255, 255, 0.35)",
                      padding: "10px 14px 4px",
                    }}
                  >
                    Songs
                  </div>
                  {tracks.map((t, i) => {
                    const itemIndex = 1 + i;
                    const isSelected = selectedIndex === itemIndex;
                    return (
                      <div
                        key={t.id}
                        data-index={itemIndex}
                        onClick={() => handleSelect(itemIndex)}
                        onMouseEnter={() => setSelectedIndex(itemIndex)}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: 12,
                          padding: "8px 12px",
                          borderRadius: 8,
                          cursor: "pointer",
                          background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
                          transition: "background 0.1s",
                        }}
                      >
                        <div style={{ position: "relative", width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                          <CoverArt url={t.album?.image_url} alt={t.name} size={34} />
                          {isSelected && (
                            <div
                              style={{
                                position: "absolute",
                                inset: 0,
                                background: "rgba(0, 0, 0, 0.4)",
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "center",
                              }}
                            >
                              <Play size={14} fill="#fff" strokeWidth={0} />
                            </div>
                          )}
                        </div>
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <p style={{ margin: 0, fontSize: 13, fontWeight: 600, color: isSelected ? "#ffffff" : "rgba(255, 255, 255, 0.9)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {t.name}
                          </p>
                          <p style={{ margin: "2px 0 0", fontSize: 11.5, color: "rgba(255, 255, 255, 0.45)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {t.artists.map((a) => a.name).join(", ")}
                          </p>
                        </div>
                        <span style={{ fontSize: 11, color: "rgba(255, 255, 255, 0.35)", fontVariantNumeric: "tabular-nums" }}>
                          {fmtMs(t.duration_ms)}
                        </span>
                      </div>
                    );
                  })}
                </>
              )}

              {/* Albums Section */}
              {albums.length > 0 && (
                <>
                  <div
                    style={{
                      fontSize: 10.5,
                      fontWeight: 700,
                      letterSpacing: "0.08em",
                      textTransform: "uppercase",
                      color: "rgba(255, 255, 255, 0.35)",
                      padding: "10px 14px 4px",
                    }}
                  >
                    Albums
                  </div>
                  {albums.map((a, i) => {
                    const itemIndex = 1 + tracks.length + i;
                    const isSelected = selectedIndex === itemIndex;
                    return (
                      <div
                        key={a.id}
                        data-index={itemIndex}
                        onClick={() => handleSelect(itemIndex)}
                        onMouseEnter={() => setSelectedIndex(itemIndex)}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: 12,
                          padding: "8px 12px",
                          borderRadius: 8,
                          cursor: "pointer",
                          background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
                          transition: "background 0.1s",
                        }}
                      >
                        <div style={{ width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}>
                          <CoverArt url={a.image_url} alt={a.name} size={34} />
                        </div>
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <p style={{ margin: 0, fontSize: 13, fontWeight: 600, color: isSelected ? "#ffffff" : "rgba(255, 255, 255, 0.9)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {a.name}
                          </p>
                          <p style={{ margin: "2px 0 0", fontSize: 11.5, color: "rgba(255, 255, 255, 0.45)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {a.artists.map((ar) => ar.name).join(", ")}
                          </p>
                        </div>
                        <span
                          style={{
                            fontSize: 10,
                            fontWeight: 650,
                            color: "rgba(255, 255, 255, 0.4)",
                            background: "rgba(255, 255, 255, 0.06)",
                            padding: "2px 6px",
                            borderRadius: 4,
                          }}
                        >
                          Album
                        </span>
                      </div>
                    );
                  })}
                </>
              )}

              {/* Artists Section */}
              {artists.length > 0 && (
                <>
                  <div
                    style={{
                      fontSize: 10.5,
                      fontWeight: 700,
                      letterSpacing: "0.08em",
                      textTransform: "uppercase",
                      color: "rgba(255, 255, 255, 0.35)",
                      padding: "10px 14px 4px",
                    }}
                  >
                    Artists
                  </div>
                  {artists.map((ar, i) => {
                    const itemIndex = 1 + tracks.length + albums.length + i;
                    const isSelected = selectedIndex === itemIndex;
                    return (
                      <div
                        key={ar.id}
                        data-index={itemIndex}
                        onClick={() => handleSelect(itemIndex)}
                        onMouseEnter={() => setSelectedIndex(itemIndex)}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          gap: 12,
                          padding: "8px 12px",
                          borderRadius: 8,
                          cursor: "pointer",
                          background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
                          transition: "background 0.1s",
                        }}
                      >
                        <div style={{ width: 34, height: 34, borderRadius: "50%", overflow: "hidden", flexShrink: 0 }}>
                          <CoverArt url={ar.image_url} alt={ar.name} size={34} />
                        </div>
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <p style={{ margin: 0, fontSize: 13, fontWeight: 600, color: isSelected ? "#ffffff" : "rgba(255, 255, 255, 0.9)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {ar.name}
                          </p>
                        </div>
                        <span
                          style={{
                            fontSize: 10,
                            fontWeight: 650,
                            color: "rgba(255, 255, 255, 0.4)",
                            background: "rgba(255, 255, 255, 0.06)",
                            padding: "2px 6px",
                            borderRadius: 4,
                          }}
                        >
                          Artist
                        </span>
                      </div>
                    );
                  })}
                 </>
               )}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
