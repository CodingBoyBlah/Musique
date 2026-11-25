import { Activity, memo, useEffect, useLayoutEffect, useRef, useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { motion, AnimatePresence, useReducedMotion, MotionGlobalConfig } from "framer-motion";
import { TitleBar as TitleBarView, WindowCaptionControls as CaptionControlsView } from "./TitleBar";
import SidebarView from "./Sidebar";
import { PlayerBar as PlayerBarView } from "./PlayerBar";
import { usePositionTicker } from "../../hooks/usePositionTicker";
import { QueuePanel } from "./QueuePanel";
import { FriendsPanel } from "./FriendsPanel";
import { LyricsPanel } from "./LyricsPanel";
import { QuitConfirm as QuitConfirmView } from "../ui/QuitConfirm";
import { Toaster as ToasterView } from "../ui/Toaster";
import { Immersive as ImmersiveView } from "./Immersive";
import { AddToPlaylistModal as AddToPlaylistView } from "../ui/AddToPlaylistModal";
import { CreditsModal as CreditsView } from "../ui/CreditsModal";
import { YtMatchModal as YtMatchView } from "../ui/YtMatchModal";
import { DevicesPopover as DevicesView } from "./DevicesPopover";
import { usePlayerStore } from "../../store/player.store";
import { useUIStore } from "../../store/ui.store";
import { getBackdropActive } from "../../api/window";
import { backdropScrim } from "../../lib/backdrop";
import { isMac } from "../../lib/platform";
import { usePrefsStore } from "../../store/prefs.store";
import { chromePx } from "../../lib/zoom";
import { EASE_OUT, RAIL_CLOSE_S, RAIL_EASE_CSS, RAIL_OPEN_S } from "../../lib/motion";
import { pinRailGrids } from "../../lib/railFlip";
import { measurePage, playPage, snapshotPage } from "../../lib/railReflow";
import "../../styles/layout.css";
import Home from "../../pages/Home";

/* The shell's fixed pieces take no props from Layout - each reads the stores
   it needs itself - so Layout re-rendering (every panel toggle, every resize
   that crosses the crush line) has nothing to tell them. Memoised, a panel
   toggle renders Layout and the panel, not the whole window, which is most of
   the work in the frame the slide starts on. */
const Sidebar = memo(SidebarView);
const TitleBar = memo(TitleBarView);
const WindowCaptionControls = memo(CaptionControlsView);
const PlayerBar = memo(PlayerBarView);
const QuitConfirm = memo(QuitConfirmView);
const Toaster = memo(ToasterView);
const Immersive = memo(ImmersiveView);
const AddToPlaylistModal = memo(AddToPlaylistView);
const CreditsModal = memo(CreditsView);
const YtMatchModal = memo(YtMatchView);
const DevicesPopover = memo(DevicesView);

/* The routed page, cut off from Layout's own renders. A bare <Outlet/>
   re-renders the whole page every time Layout does - a panel toggle, a
   play/pause, a new cover tint - because each Layout render hands the page a
   fresh route context. On an album that was every track row and tooltip
   rendering on the first frame of a panel slide. Memoised, the page renders
   when its route does and when its own state changes, never for the shell. */
const PageOutlet = memo(function PageOutlet() {
  return <Outlet />;
});

/* Home, kept alive.

   Home is the screen people come back to most, and it is the heaviest one to
   build: every shelf, tile and carousel. Mounted fresh on each return it was a
   70-90ms task (React building ~2000 elements, framer registering every tile,
   each carousel measuring itself) and then a full paint - six or seven dropped
   frames every time you came back to it.

   So it is never unmounted. Away from it, <Activity mode="hidden"> hides it
   and unmounts its effects (no store or query subscriptions, no resize
   pulses), keeping its React state and DOM. Coming back re-runs the effects -
   query subscriptions included, so stale data refetches exactly as it did on
   a fresh mount - and skips rebuilding the tree. The entrance is the same one
   every routed page plays (opacity and a 6px rise, 180ms), run here as a Web
   Animation each time Home is shown. */
/* Whatever the pointer is over when Home is hidden never gets its leave
   events - it simply stops being displayed - so a tile clicked to open its
   page would come back still hovered (lifted, highlighted). A fresh mount
   never had that problem, so the hovered element is sent the leave events it
   is about to miss, at the moment the app navigates away from Home: before
   React hides it (React ignores events during a commit, and framer has taken
   its hover listeners down by then), in the same task as the navigation, so
   nothing un-hovers on screen. React synthesises enter/leave from mouseout;
   framer's whileHover listens for pointerleave on each element. */
let homeHover: Element | null = null;
function releaseHomeHover() {
  const t = homeHover;
  homeHover = null;
  if (!t || !t.isConnected) return;
  const out = { bubbles: true, relatedTarget: document.body };
  // the lift settles at once: Home is hidden before a spring back could run,
  // and a cut-off spring would leave the tile raised for the next visit
  const instant = MotionGlobalConfig.instantAnimations;
  MotionGlobalConfig.instantAnimations = true;
  try {
    t.dispatchEvent(new MouseEvent("mouseout", out));
    t.dispatchEvent(new PointerEvent("pointerout", out));
    for (let el: Element | null = t; el && !el.matches("main"); el = el.parentElement) {
      el.dispatchEvent(new PointerEvent("pointerleave", { relatedTarget: document.body }));
    }
  } finally {
    MotionGlobalConfig.instantAnimations = instant;
  }
  /* framer settles the value only on its next frame, by which time the tile
     is hidden and unbound, so its cached value keeps the lift. The tile is
     settled again once Home is back (settleHomeHover) - framer is bound then. */
  staleCard = t.closest<HTMLElement>(".card-link");
}
let staleCard: HTMLElement | null = null;
// runs as Home is shown again, before its first frame (still at opacity 0)
function settleHomeHover() {
  const card = staleCard;
  staleCard = null;
  if (!card || !card.isConnected) return;
  const instant = MotionGlobalConfig.instantAnimations;
  MotionGlobalConfig.instantAnimations = true;
  try {
    card.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "mouse" }));
    card.dispatchEvent(new PointerEvent("pointerleave", { pointerType: "mouse", relatedTarget: document.body }));
  } finally {
    MotionGlobalConfig.instantAnimations = instant;
  }
}
if (typeof window !== "undefined") {
  const leavingHome = (url: unknown) => {
    if (location.pathname !== "/" || url == null) return;
    try {
      if (new URL(String(url), location.href).pathname !== "/") releaseHomeHover();
    } catch { /* not a URL we navigate to */ }
  };
  for (const k of ["pushState", "replaceState"] as const) {
    const orig = history[k];
    history[k] = function (this: History, data: unknown, unused: string, url?: string | URL | null) {
      leavingHome(url);
      return orig.call(this, data, unused, url);
    };
  }
  // back / forward: runs before the router's own listener (registered later)
  let lastPath = location.pathname;
  window.addEventListener("popstate", () => {
    if (lastPath === "/" && location.pathname !== "/") releaseHomeHover();
    lastPath = location.pathname;
  }, true);
  const track = () => { lastPath = location.pathname; };
  window.addEventListener("popstate", track);
  for (const k of ["pushState", "replaceState"] as const) {
    const orig = history[k];
    history[k] = function (this: History, data: unknown, unused: string, url?: string | URL | null) {
      const r = orig.call(this, data, unused, url);
      track();
      return r;
    };
  }
}

const HomeStage = memo(function HomeStage() {
  const ref = useRef<HTMLDivElement>(null);
  const reduceMotion = useReducedMotion();

  // layout effects re-run every time the Activity is shown again
  useLayoutEffect(() => {
    const el = ref.current;
    // shelves open at their start, as they did when Home was built fresh
    el?.querySelectorAll<HTMLElement>(".edge-fade-x").forEach((c) => { if (c.scrollLeft) c.scrollLeft = 0; });
    settleHomeHover();
    if (!el || reduceMotion || typeof el.animate !== "function") return;
    const anim = el.animate(
      [{ opacity: 0, transform: "translateY(6px) scale(0.995)" }, { opacity: 1, transform: "none" }],
      { duration: 180, easing: `cubic-bezier(${EASE_OUT.join(", ")})` },
    );
    return () => anim.cancel();
  }, [reduceMotion]);
  return (
    <div ref={ref} onPointerOver={(e) => { homeHover = e.target as Element; }}>
      <Home />
    </div>
  );
});

/* collapsed sidebar. on mac the native traffic lights live in this column at
   their fixed OS positions (12px dots, 20px pitch, first centre at x=20), so
   the collapsed rail has to stay wide enough to hold all three. */
const COLLAPSED_SIDEBAR_W = 64;

/* the right rail's panels and the room each takes */
type PanelKey = "lyrics" | "queue" | "friends";
const PANEL_W: Record<PanelKey, number> = { lyrics: 366, queue: 272, friends: 272 };

/** An element's current horizontal translation, running animations included. */
function translateXOf(el: HTMLElement): number {
  const t = getComputedStyle(el).transform;
  return !t || t === "none" ? 0 : new DOMMatrixReadOnly(t).m41;
}

// what sequential focus navigation stops on
const TABBABLE =
  'a[href],button:not([disabled]),input:not([disabled]):not([type="hidden"]),select:not([disabled]),textarea:not([disabled]),[tabindex],[contenteditable]:not([contenteditable="false"]),audio[controls],video[controls],summary';
const MAC_COLLAPSED_SIDEBAR_W = 72;

/* the page's cover-art wash: concentrated behind the header (cover, title,
   buttons), then a long ease-out tail that thins to nothing toward the
   bottom. many stops on a quadratic curve so the fade has no visible edge. */
const BLOOM_MASK =
  "linear-gradient(180deg, #000 0%, rgba(0,0,0,0.96) 12%, rgba(0,0,0,0.8) 24%, " +
  "rgba(0,0,0,0.58) 34%, rgba(0,0,0,0.38) 44%, rgba(0,0,0,0.23) 54%, " +
  "rgba(0,0,0,0.12) 64%, rgba(0,0,0,0.05) 76%, rgba(0,0,0,0.015) 88%, transparent 100%)";

export default function Layout() {
  const queueOpen = usePlayerStore((s) => s.queueOpen);
  const lyricsOpen = usePlayerStore((s) => s.lyricsOpen);
  // the panel can be switched off in settings; a stale open flag must not keep it
  const friendsFlag = usePlayerStore((s) => s.friendsOpen);
  const friendsOn = usePrefsStore((s) => s.showFriends);
  const friendsOpen = friendsFlag && friendsOn;
  const immersiveOpen = usePlayerStore((s) => s.immersiveOpen);
  // one owner for the playhead clock, whatever else is on screen
  usePositionTicker();
  const effect = useUIStore((s) => s.windowEffect);
  const materialTransparency = useUIStore((s) => s.materialTransparency);
  const pageTint = useUIStore((s) => s.pageTint);
  const backdropActive = useUIStore((s) => s.backdropActive);
  const setBackdropActive = useUIStore((s) => s.setBackdropActive);
  const location = useLocation();
  // Home lives outside the routed outlet (HomeStage)
  const isHome = location.pathname === "/";
  const reduceMotion = useReducedMotion();
  const mainRef = useRef<HTMLElement>(null);

  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const macSimulated = useUIStore((s) => s.macSimulated);
  const uiZoom = usePrefsStore((s) => s.uiZoom);
  // same sum as the Sidebar's rail: the traffic lights' room in physical px
  const collapsedSidebarW = (isMac || macSimulated)
    ? Math.max(COLLAPSED_SIDEBAR_W, chromePx(MAC_COLLAPSED_SIDEBAR_W, uiZoom))
    : COLLAPSED_SIDEBAR_W;
  const [willCrushMain, setWillCrushMain] = useState(false);
  const shellRef = useRef<HTMLDivElement>(null);

  const immersiveCovered = useUIStore((s) => s.immersiveCovered);
  const setImmersiveCovered = useUIStore((s) => s.setImmersiveCovered);

  /* The shell under the immersive overlay.

     Once the overlay has fully faded in, the shell stops rendering
     (content-visibility: hidden). It matters: the lyric sweep repaints a line
     every frame, and every repaint re-layerizes the whole document - with the
     shell still painted underneath that doubled the renderer's CPU for as
     long as the view stayed up.

     It used to be hidden with visibility (and inerted). Both are inherited,
     so each flip restyled all ~1500 elements of the shell, and inert changes
     the hit-test data recorded in paint, so it repainted it too: a 20-26ms
     frame on the way in and another on the frame the exit fade started.
     content-visibility is not inherited (one element restyles), keeps the
     shell's style, layout and scroll positions, and takes its contents out of
     focus order. It comes back in the same commit the exit fade starts in;
     the overlay is still opaque there, and its tiles raster under the fade.

     The timer is only a backstop in case the overlay's signal never arrives -
     it is well past the 260ms fade, so it never races the real one. */
  const shellHidden = immersiveOpen && immersiveCovered;
  useEffect(() => {
    if (!immersiveOpen) {
      if (immersiveCovered) setImmersiveCovered(false);
      return;
    }
    if (immersiveCovered) return;
    const t = setTimeout(() => setImmersiveCovered(true), 700);
    return () => clearTimeout(t);
  }, [immersiveOpen, immersiveCovered, setImmersiveCovered]);

  /* Tab, before the shell is hidden (the fade in) - content-visibility takes
     care of it after. Focus moves through the focusable elements outside the
     shell in document order, wrapping at either end: the order the browser
     gives once the shell is out of it. On document, in the capture phase, so a
     modal's own trap (on window) runs first; whatever it has already handled
     is left alone. Steering Tab costs nothing; inerting the shell for this
     restyled and repainted all of it. */
  useEffect(() => {
    const shell = shellRef.current;
    if (!immersiveOpen || !shell) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Tab" || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
      const order = Array.from(document.querySelectorAll<HTMLElement>(TABBABLE)).filter(
        (el) => el.tabIndex >= 0 && !shell.contains(el) && !el.closest("[inert]") && el.getClientRects().length > 0,
      );
      e.preventDefault();
      if (!order.length) return;
      const at = order.indexOf(document.activeElement as HTMLElement);
      const next = at === -1
        ? (e.shiftKey ? order[order.length - 1] : order[0])
        : order[(at + (e.shiftKey ? -1 : 1) + order.length) % order.length];
      next.focus();
    };
    document.addEventListener("keydown", onKey, true);
    return () => document.removeEventListener("keydown", onKey, true);
  }, [immersiveOpen]);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const checkCrush = () => {
      const w = window.innerWidth;
      const isCollapsed = sidebarCollapsed || w < 768;
      const sw = isCollapsed ? collapsedSidebarW : 232;
      const rpw = lyricsOpen ? PANEL_W.lyrics : queueOpen || friendsOpen ? PANEL_W.queue : 0;
      const nextCrush = w - sw - rpw < 340;
      setWillCrushMain((prev) => (prev === nextCrush ? prev : nextCrush));
    };

    checkCrush();

    let rId = 0;
    const onResizeThrottled = () => {
      if (rId) return;
      rId = requestAnimationFrame(() => {
        rId = 0;
        checkCrush();
      });
    };
    window.addEventListener("resize", onResizeThrottled, { passive: true });

    return () => {
      window.removeEventListener("resize", onResizeThrottled);
      if (rId) cancelAnimationFrame(rId);
    };
  }, [sidebarCollapsed, collapsedSidebarW, lyricsOpen, queueOpen, friendsOpen]);

  const rawPanelWidth = lyricsOpen ? PANEL_W.lyrics : queueOpen || friendsOpen ? PANEL_W.queue : 0;
  const spacerWidth = willCrushMain ? 0 : rawPanelWidth;
  // the panel the rail should be showing (none while the page would be crushed)
  const want: PanelKey | null = spacerWidth > 0
    ? (lyricsOpen ? "lyrics" : queueOpen ? "queue" : friendsOpen ? "friends" : null)
    : null;

  /* The right rail slides; the page changes size once.

     Animating the rail's width moved the card's edge by re-laying out and
     repainting the whole page every frame - on Home that is ~860 clipped
     boxes, and it is what made the slide stutter. Now the layout changes in
     exactly one frame: at the start of a close (the page widens under the
     panel, which then slides off it) and at the end of an open (the panel has
     slid over the page, and the page settles to the room that is left).

     Everything that moves in between is a Web Animation of a transform, on
     the compositor, started together so they share one clock:
     - the card's right edge: `mover` clips the card and slides left while
       `counter`, inside it, slides the card back by the same amount - a clip
       edge that travels while the page under it stays still. `cap` draws the
       rounded corner and hairline the clip cuts off, `shadow` is scaled to
       the visible width;
     - the docked player bar and the cover tint, re-centred on what is left;
     - the panels, in from / out to the right on their wrappers.
     Tile grids are pinned to their landing width at the start and FLIPped
     there on the same curve (lib/railFlip).

     `landed` is the rail's width at rest; React renders the resting styles
     from it, and the slide writes over them only while it runs. */
  const [landed, setLanded] = useState(spacerWidth);
  const [prevWant, setPrevWant] = useState(want);
  const [leaving, setLeaving] = useState<PanelKey[]>([]);
  if (want !== prevWant) {
    // adjusting state during render: the outgoing panel stays mounted (and
    // slides out) until the slide lands
    setPrevWant(want);
    const next = leaving.filter((k) => k !== want);
    if (prevWant && !next.includes(prevWant)) next.push(prevWant);
    setLeaving(next);
  }
  const panelKeys: PanelKey[] = want ? [...leaving.filter((k) => k !== want), want] : leaving;

  const spacerRef = useRef<HTMLDivElement>(null);
  const slotRef = useRef<HTMLDivElement>(null);
  const moverRef = useRef<HTMLDivElement>(null);
  const counterRef = useRef<HTMLDivElement>(null);
  const capRef = useRef<HTMLDivElement>(null);
  const shadowRef = useRef<HTMLDivElement>(null);
  const dockRef = useRef<HTMLDivElement>(null);
  const tintRef = useRef<HTMLDivElement>(null);
  const panelEls = useRef(new Map<PanelKey, HTMLDivElement>());
  const landedRef = useRef(spacerWidth);
  const shownRef = useRef<PanelKey | null>(want);
  // the running slide; `edge` when it moves the card's edge (not a same-width panel swap)
  const slideRef = useRef<{ anims: Animation[]; unpin: () => void; edge: boolean } | null>(null);
  const slideToken = useRef(0);
  const reduceRef = useRef(reduceMotion);
  reduceRef.current = reduceMotion;

  useLayoutEffect(() => {
    const to = spacerWidth;
    const entering = want;
    if (!slideRef.current && to === landedRef.current && entering === shownRef.current) return;
    shownRef.current = entering;
    const token = ++slideToken.current;

    // after the commit, before the frame is painted: the JS-columned grids
    // re-render synchronously inside the pin, which React refuses mid-commit
    queueMicrotask(() => {
      if (token !== slideToken.current) return;
      const els = [spacerRef, slotRef, moverRef, counterRef, capRef, shadowRef, dockRef, tintRef, mainRef].map((r) => r.current);
      if (els.some((el) => !el)) return;
      const [spacer, slot, mover, counter, cap, shadow, dock, tint, main] = els as HTMLElement[];
      const prev = slideRef.current;
      const panels = [...panelEls.current];

      // where everything is on screen right now (mid-slide included: computed
      // style reads through running animations)
      const r0 = prev?.edge ? -translateXOf(mover) : landedRef.current;
      const x0 = new Map(panels.map(([k, el]) => [k, translateXOf(el)]));

      const land = (unpin: () => void, anims: Animation[]) => {
        spacer.style.width = `${to}px`;
        tint.style.right = `${-to}px`;
        tint.style.transform = `translateX(${-to / 2}px)`;
        dock.style.right = "0px";
        main.style.overflowAnchor = "";
        delete slot.dataset.sliding;
        // resting transforms written before the animations are dropped, so
        // nothing shows its pre-slide style for a frame
        for (const [k, el] of panelEls.current) el.style.transform = k === entering ? "" : `translateX(${PANEL_W[k]}px)`;
        for (const a of anims) a.cancel();
        unpin();
        slideRef.current = null;
        landedRef.current = to;
        setLanded(to);
        setLeaving([]);
      };

      if (reduceRef.current) {
        if (prev) for (const a of prev.anims) a.cancel();
        prev?.unpin();
        land(() => {}, []);
        return;
      }

      const ms = (to > r0 ? RAIL_OPEN_S : RAIL_CLOSE_S) * 1000;
      const opts: KeyframeAnimationOptions = { duration: ms, easing: RAIL_EASE_CSS, fill: "forwards" };
      const slideX = (el: HTMLElement, from: number, end: number) =>
        el.animate([{ transform: `translateX(${from}px)` }, { transform: `translateX(${end}px)` }], opts);
      const slidePanels = () =>
        panels.map(([k, el]) => slideX(el, x0.get(k) ?? PANEL_W[k], k === entering ? 0 : PANEL_W[k]));

      if (!prev?.edge && r0 === to) {
        // one panel for another of the same width (queue <-> friends): the
        // card's edge stays where it is, so the page is not touched at all
        if (prev) for (const a of prev.anims) a.cancel();
        const anims = slidePanels();
        slideRef.current = { anims, unpin: () => {}, edge: false };
        anims[0]?.finished.then(() => {
          if (token === slideToken.current) land(() => {}, anims);
        }, () => {});
        if (!anims.length) land(() => {}, []);
        return;
      }

      // where the page is on screen before anything changes (lib/railReflow)
      const page = snapshotPage(main);
      const unpin = pinRailGrids(main, -to, ms, () => {
        // the one layout change at the start of a slide: the rail gives its
        // room back to the card for the length of the slide (a no-op when it
        // was already closed). The card's clip takes over the visible edge.
        if (prev) for (const a of prev.anims) a.cancel();
        prev?.unpin();
        spacer.style.width = "0px";
        tint.style.right = "0px";
        // the player bar takes its landing width now (it is narrower than
        // 740px only in a small window) and glides to the centre of the card
        // that is left, rather than re-sizing when the slide lands
        dock.style.right = `${to}px`;
        // the page is measured in two layouts below; scroll anchoring would
        // adjust between them differently from the real landing
        main.style.overflowAnchor = "none";
        slot.dataset.sliding = "";
      });
      measurePage(page, "wide");
      if (to > 0) {
        // the page as it will be once the rail lands, read and put back in
        // the same frame
        const top = main.scrollTop;
        main.style.right = `${to}px`;
        measurePage(page, "landing");
        main.style.right = "0px";
        main.scrollTop = top;
      } else {
        for (const n of page.nodes) n.landing = n.wide;
      }
      const w = slot.getBoundingClientRect().width || 1;
      const anims = [
        slideX(mover, -r0, -to),
        slideX(counter, r0, to),
        slideX(cap, -r0, -to),
        shadow.animate([{ transform: `scaleX(${(w - r0) / w})` }, { transform: `scaleX(${(w - to) / w})` }], opts),
        slideX(dock, (to - r0) / 2, 0),
        slideX(tint, -r0 / 2, -to / 2),
        ...slidePanels(),
        ...playPage(page, opts),
      ];
      slideRef.current = { anims, unpin, edge: true };
      anims[0].finished.then(() => {
        if (token === slideToken.current) land(unpin, anims);
      }, () => {});
    });
  }, [spacerWidth, want]);

  useEffect(() => () => {
    slideToken.current++;
    slideRef.current?.anims.forEach((a) => a.cancel());
  }, []);

  /* `trim_memory` used to fire 1.5s after every navigation. That call empties
     the working set of this process and of every WebView2 child, so it landed
     right as the user was reading and scrolling the page they had just opened,
     and everything had to be faulted back in. Genuine idle trimming now happens
     in the native layer, only once the window has actually been out of sight
     for a while (see mem_trim.rs). */
  useEffect(() => {
    if (mainRef.current) mainRef.current.scrollTop = 0;
  }, [location.pathname]);

  useEffect(() => {
    getBackdropActive().then(setBackdropActive).catch(() => setBackdropActive(false));
  }, [setBackdropActive]);

  const toggleMacSimulated = useUIStore((s) => s.toggleMacSimulated);

  // Ctrl+K lives with the search field in the top bar (TopSearch).
  // Ctrl+Shift+M (fake mac chrome) is a dev-only layout aid
  useEffect(() => {
    if (!import.meta.env.DEV) return;
    function onGlobalKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "m") {
        e.preventDefault();
        toggleMacSimulated();
      }
    }
    window.addEventListener("keydown", onGlobalKey);
    return () => window.removeEventListener("keydown", onGlobalKey);
  }, [toggleMacSimulated]);

  // backdrop strategy:
  //  - no live material (Linux, or Mica/vibrancy failed) -> paint the solid app
  //    bg, otherwise the transparent window shows white (or the desktop)
  //  - windows acrylic -> OS ignores the tint, so darken with a CSS scrim
  //  - windows Mica / macOS vibrancy -> stay transparent, OS material shows
  const scrim = backdropScrim(backdropActive, effect, materialTransparency, isMac);
  const isTranslucent = backdropActive && effect !== "none";
  const cardBg = isTranslucent
    ? `rgba(18, 18, 20, ${Math.max(0.66, Math.min(0.85, 0.78 * (1 - materialTransparency * 0.28)))})`
    : "rgba(19, 19, 19, 0.94)";

  return (
    /*
     * window root is fully transparent - the OS Mica material (applied in rust)
     * is the background. no backdrop-filter here, that'd blur the desktop a
     * second time and fight Mica. win11 rounds the frame.
     */
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        overflow: "hidden",
        background: scrim,
        transition: "background 0.2s",
      }}
    >
      <div
        style={{
          display: "flex",
          flex: 1,
          minHeight: 0,
          overflow: "hidden",
          position: "relative",
          contentVisibility: shellHidden ? "hidden" : undefined,
        }}
        ref={shellRef}
      >
        {/* the sidebar runs the full height of the window */}
        <Sidebar />

        {/* everything right of the sidebar: the top bar on the OS material,
            and under it the page card plus the lyrics / queue rail */}
        <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
          <div style={{ flexShrink: 0, position: "relative", zIndex: 20 }}>
            <TitleBar />
          </div>

          <div style={{ flex: 1, minHeight: 0, display: "flex", position: "relative" }}>
            {/* The Main Window Island Card. The slot is its box in the row;
                the mover / counter pair, the cap and the shadow are how its
                right edge travels during a rail slide (see "The right rail
                slides" above). At rest they carry no transform and the card
                is exactly the card. */}
            <div ref={slotRef} className="card-slot" style={{ flex: 1, minWidth: 0, margin: "0 4px 4px", position: "relative" }}>
              {/* negative spread keeps the lift underneath the card instead
                  of letting it bleed up into the top bar. Its own element so
                  a slide can scale it to the visible card */}
              <div
                ref={shadowRef}
                aria-hidden
                style={{
                  position: "absolute",
                  inset: 0,
                  borderRadius: 12,
                  boxShadow: "0 14px 30px -8px rgba(0, 0, 0, 0.55)",
                  transformOrigin: "left center",
                  pointerEvents: "none",
                }}
              />
              <div ref={moverRef} className="card-mover">
                <div ref={counterRef} className="card-counter">
            <div
              style={{
                position: "absolute",
                inset: 0,
                borderRadius: 12,
                border: "1px solid rgba(255, 255, 255, 0.08)",
                background: "transparent",
                overflow: "hidden",
                display: "flex",
                flexDirection: "column",
              }}
            >
              {/* the card's material, on its own layer under the content.

                  NO backdrop-filter here, deliberately. nothing is ever painted
                  between the window root and this card, so it would only blur
                  transparency (or, with no live material, a flat scrim colour)
                  while costing a compositing surface of its own. */}
              <div
                aria-hidden
                style={{
                  position: "absolute",
                  inset: 0,
                  zIndex: 0,
                  pointerEvents: "none",
                  borderRadius: 12,
                  background: cardBg,
                  boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.08)",
                }}
              />

              {/* Cover art bloom contained within the island card.
                  Blurred, so it must never be resized mid-slide - that re-runs
                  the blur every frame. It stays the width of card + rail (a
                  constant), overhangs the card's edge where the card clips
                  it, and is moved back to the card's centre by a transform,
                  which costs nothing. */}
              <div
                ref={tintRef}
                aria-hidden
                style={{
                  position: "absolute",
                  top: 0, left: 0, bottom: 0,
                  right: -landed,
                  transform: landed ? `translateX(${-landed / 2}px)` : undefined,
                  zIndex: 0,
                  pointerEvents: "none",
                }}
              >
              <AnimatePresence>
                {pageTint && (
                  <motion.div
                    key={pageTint}
                    aria-hidden
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 0.4 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.7, ease: "easeOut" }}
                    style={{
                      position: "absolute",
                      inset: 0,
                      pointerEvents: "none",
                      overflow: "hidden",
                      borderRadius: 12,
                    }}
                  >
                    <div
                      style={{
                        position: "absolute",
                        inset: 0,
                        backgroundImage: `url(${pageTint})`,
                        backgroundSize: "cover",
                        backgroundPosition: "center top",
                        /* brightness pulls pale covers down so white type
                           on top keeps its contrast at the higher opacity */
                        filter: "blur(80px) saturate(1.6) brightness(0.85)",
                        transform: "scale(1.6)",
                        transformOrigin: "center top",
                        /* a full-width wash that eases out down the page
                           (many stops so the fade has no visible edge),
                           rather than a spotlight in the middle */
                        maskImage: BLOOM_MASK,
                        WebkitMaskImage: BLOOM_MASK,
                      }}
                    />
                    {/* grain breaks up the banding a smooth dark gradient
                        gets on 8-bit panels */}
                    <div
                      style={{
                        position: "absolute",
                        inset: 0,
                        maskImage: BLOOM_MASK,
                        WebkitMaskImage: BLOOM_MASK,
                        opacity: 0.07,
                        mixBlendMode: "overlay",
                        backgroundRepeat: "repeat",
                        backgroundImage:
                          "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E\")",
                      }}
                    />
                  </motion.div>
                )}
              </AnimatePresence>
              </div>

              {/* Scrolling page view inside card */}
              <div style={{ position: "relative", flex: 1, minHeight: 0, overflow: "hidden" }}>
                <main
                  ref={mainRef}
                  style={{
                    position: "absolute",
                    inset: 0,
                    overflowY: "auto",
                    overflowX: "hidden",
                    paddingTop: 20,
                    paddingLeft: "clamp(14px, 2.5vw, 32px)",
                    paddingRight: "clamp(14px, 2.5vw, 32px)",
                    paddingBottom: "90px",
                  }}
                >
                  {/* `transform` as one string, not framer's y/scale: those
                      are composed in JS and written every frame on the main
                      thread - right while the new page mounts - where a
                      whole transform runs as a Web Animation on the
                      compositor. Same values, same curve. Back to "none" at
                      rest (transitionEnd) - framer reads a "none" target as
                      scale(0), and any leftover transform would make this the
                      containing block for the pages' unportalled modals. */}
                  <Activity mode={isHome ? "visible" : "hidden"}>
                    <HomeStage />
                  </Activity>
                  {!isHome && (
                    <motion.div
                      key={location.pathname}
                      initial={reduceMotion ? false : { opacity: 0, transform: "translateY(6px) scale(0.995)" }}
                      animate={{ opacity: 1, transform: "translateY(0px) scale(1)", transitionEnd: { transform: "none" } }}
                      transition={{ duration: 0.18, ease: EASE_OUT }}
                    >
                      <PageOutlet />
                    </motion.div>
                  )}
                </main>
              </div>

              {/* PlayerBar docked inside card. The immersive view floats its own
                  copy over the top, so this one stands down rather than render and
                  paint underneath an opaque overlay. */}
              <div ref={dockRef} style={{ position: "absolute", inset: 0, zIndex: 40, pointerEvents: "none" }}>
                {!immersiveOpen && <PlayerBar />}
              </div>
            </div>
                </div>
              </div>
              {/* the rounded corner and hairline of the card's right edge
                  while the mover's clip stands in for it */}
              <div ref={capRef} className="card-cap" aria-hidden />
            </div>

            {/* Right rail. The spacer is the room the panel takes in the row
                (its width only changes at rest - see "The right rail
                slides"); the panels themselves ride over the row's right
                edge on wrappers the slide moves. */}
            <div ref={spacerRef} style={{ width: landed, flexShrink: 0 }} />
            <div
              style={{
                position: "absolute",
                top: 0,
                right: 0,
                bottom: 0,
                width: PANEL_W.lyrics,
                overflow: "hidden",
                pointerEvents: "none",
              }}
            >
              {panelKeys.map((k) => (
                <div
                  key={k}
                  ref={(el) => {
                    if (el) panelEls.current.set(k, el);
                    else panelEls.current.delete(k);
                  }}
                  style={{
                    position: "absolute",
                    top: 0,
                    right: 0,
                    bottom: 0,
                    width: PANEL_W[k],
                    pointerEvents: "auto",
                    // a panel mounts off the edge; the slide brings it in
                    transform: k === shownRef.current ? undefined : `translateX(${PANEL_W[k]}px)`,
                  }}
                >
                  {k === "lyrics" ? <LyricsPanel /> : k === "queue" ? <QueuePanel /> : <FriendsPanel />}
                </div>
              ))}
            </div>
          </div>
        </div>

      </div>

      <DevicesPopover />
      <Immersive />
      <QuitConfirm />
      <AddToPlaylistModal />
      <CreditsModal />
      <YtMatchModal />
      <Toaster />
      {/* always at the window's own top-right corner, in the top bar's strip,
          so the buttons sit on the OS material and the close button owns the
          corner pixel */}
      <WindowCaptionControls />
    </div>
  );
}
