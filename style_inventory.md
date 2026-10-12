# Style and motion inspection inventory

Parsed from every TypeScript syntax tree. Static appearance is preserved; these are review references, not proposed changes.

## src/components/ErrorBoundary.tsx

```text
41 fontSize = 20
41 fontWeight = 700
42 fontSize = 13.5
42 lineHeight = 1.5
49 fontSize = 12
61 fontSize = 13.5
61 fontWeight = 600
```

## src/components/UpdatePrompt.tsx

```text
40 boxShadow = "0 24px 70px rgba(0,0,0,0.55)"
78 fontSize = 12.5
79 lineHeight = 1.5
115 initial={false}
116 animate={{ scaleX: progress / 100 }}
117 transition={{ ease: EASE_OUT, duration: 0.25 }}
129 fontSize = 12
131 letterSpacing = "0.004em"
184 fontSize = 17
185 fontWeight = 700
186 letterSpacing = "-0.01em"
190 fontSize = 13
192 lineHeight = 1.45
213 fontSize = 13.5
```

## src/components/layout/DevicesPopover.tsx

```text
144 initial={{ opacity: 0, y: below ? -6 : 6, scale: 0.96 }}
145 animate={{ opacity: 1, y: 0, scale: 1 }}
146 exit={{ opacity: 0, y: below ? -4 : 4, scale: 0.97, transition: { duration: 0.12, ease: EASE_OUT } }}
147 transition={{ duration: 0.18, ease: EASE_OUT }}
161 backdropFilter = "blur(32px)"
162 WebkitBackdropFilter = "blur(32px)"
165 boxShadow = "0 18px 48px rgba(0, 0, 0, 0.65), 0 2px 10px rgba(0, 0, 0, 0.35)"
184 fontSize = 12.5
185 fontWeight = 600
238 fontSize = 12
307 fontSize = 12.5
308 fontWeight = isActive ? 600 : 500
322 fontSize = 10.5
352 boxShadow = "0 0 6px rgba(255, 255, 255, 0.5)"
```

## src/components/layout/FriendsPanel.tsx

```text
65 fontSize = 10.5
65 fontWeight = 700
65 letterSpacing = "0.08em"
85 lineHeight = 0
89 fontSize = 15
89 fontWeight = 700
102 fontSize = 13
102 fontWeight = 650
106 fontSize = 11
110 fontSize = 12
117 fontSize = 11.5
122 fontSize = 11.5
163 initial={{ x: WIDTH }}
164 animate={{ x: 0 }}
165 exit={{ x: WIDTH, transition: RAIL_CLOSE }}
166 transition={RAIL_OPEN}
167 willChange = "transform"
170 fontSize = 14
170 fontWeight = 700
170 letterSpacing = "-0.01em"
174 animate={{ rotate: isFetching ? 360 : 0 }}
174 transition={isFetching ? { repeat: Infinity, duration: 0.9, ease: "linear" } : { duration: 0 }}
198 fontSize = 13
198 fontWeight = 600
199 fontSize = 12
199 lineHeight = 1.45
```

## src/components/layout/Immersive.tsx

```text
104 initial={{ opacity: 0 }}
105 animate={{ opacity: shown ? 1 : 0 }}
106 transition={{ duration: 0.6, ease: [0.4, 0, 0.2, 1] }}
142 contain = "strict"
146 initial={false}
150 initial={{ opacity: 0 }}
151 animate={{ opacity: 1 }}
153 exit={{ opacity: 0, transition: { delay: ROOM_FADE_S, duration: 0 } }}
154 transition={{ duration: ROOM_FADE_S, ease: [0.4, 0, 0.2, 1] }}
188 backgroundImage = `url("${a.art}")`
192 backgroundImage = `radial-gradient(circle at 34% 40%, rgba(${a.blobs[0]}, 0.75) 0%, rgba(${a.blobs[1]}, 0.35) 52%, rgba(${a.blobs[2]}, 0) 78%)`
250 initial={false}
263 initial={{ opacity: 0 }}
264 animate={{ opacity: 1 }}
265 exit={{ opacity: 0 }}
266 transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
282 initial={{ opacity: 0 }}
283 animate={{ opacity: 1 }}
284 exit={{ opacity: 0 }}
285 transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
490 fontSize = "1.07rem"
517 fontSize = 12
517 fontWeight = 700
517 letterSpacing = "0.06em"
530 fontSize = 14
530 fontWeight = 600
531 fontSize = 12.5
581 initial={{ opacity: 0 }}
582 animate={{ opacity: 1 }}
584 exit={{ opacity: 0, transition: { duration: 0.18, ease: EASE_OUT } }}
585 transition={{ duration: 0.26, ease: [0.22, 1, 0.36, 1] }}
638 initial={{ opacity: 0 }}
639 animate={{ opacity: 1 }}
642 exit={{ opacity: 0, y: 10, transition: { duration: 0.18, ease: EASE_OUT } }}
643 transition={{ duration: 0.28, delay: 0.04, ease: [0.22, 1, 0.36, 1] }}
656 boxShadow = "inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 30px 80px rgba(0, 0, 0, 0.30)"
680 initial={false}
685 initial="enter"
686 animate="center"
687 exit="exit"
688 transition={{ duration: 0.15, ease: EASE_OUT }}
713 initial={{ opacity: 0, y: 14 }}
714 animate={{ opacity: 1, y: 0 }}
716 exit={{ opacity: 0, y: 14, transition: { duration: 0.18, ease: EASE_OUT } }}
717 transition={{ duration: 0.3, delay: 0.06, ease: [0.22, 1, 0.36, 1] }}
```

## src/components/layout/JamCard.tsx

```text
33 boxShadow = ring ? "0 0 0 2px var(--color-popover, #1c1c22)" : undefined
39 fontSize = size * 0.4
40 fontWeight = 700
61 fontSize = 11.5
135 fontSize = 13
135 fontWeight = 700
138 fontSize = 11.5
142 initial={false}
146 initial={{ opacity: 0, height: 0 }}
147 animate={{ opacity: 1, height: "auto" }}
148 exit={{ opacity: 0, height: 0 }}
149 transition={{ duration: 0.18, ease: EASE_OUT }}
155 fontSize = 12.5
158 fontSize = 11
176 fontSize = 12
191 fontSize = 11.5
204 whileTap={PRESS}
205 transition={PRESS_TRANSITION}
207 fontSize = 12.5
223 fontSize = 12.5
239 fontSize = 13
239 fontWeight = 700
240 fontSize = 11.5
246 initial={false}
250 initial={{ opacity: 0, y: 4 }}
251 animate={{ opacity: 1, y: 0 }}
252 exit={{ opacity: 0, y: -4 }}
253 transition={{ duration: 0.16, ease: EASE_OUT }}
280 fontSize = 12.5
289 initial={{ opacity: 0, y: 4 }}
290 animate={{ opacity: 1, y: 0 }}
291 exit={{ opacity: 0, y: -4 }}
292 transition={{ duration: 0.16, ease: EASE_OUT }}
299 whileTap={PRESS}
300 transition={PRESS_TRANSITION}
313 fontSize = 12.5
317 fontSize = 12.5
```

## src/components/layout/Layout.tsx

```text
269 boxShadow = "0 14px 30px -8px rgba(0, 0, 0, 0.55)"
291 boxShadow = "inset 0 1px 0 rgba(255, 255, 255, 0.08)"
301 initial={{ opacity: 0 }}
302 animate={{ opacity: 0.18 }}
303 exit={{ opacity: 0 }}
304 transition={{ duration: 0.7, ease: "easeOut" }}
325 backgroundImage = `url(${pageTint})`
328 filter = "blur(72px) saturate(1.7)"
344 backgroundImage = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E\")"
369 initial={reduceMotion ? false : { opacity: 0, y: 6, scale: 0.995 }}
370 animate={{ opacity: 1, y: 0, scale: 1 }}
371 transition={{ duration: 0.18, ease: EASE_OUT }}
395 initial={false}
398 initial={false}
401 initial={false}
```

## src/components/layout/LyricReturnPill.tsx

```text
18 initial={{ opacity: 0, y: 6 }}
19 animate={{ opacity: 1, y: 0 }}
20 exit={{ opacity: 0, y: 6, transition: { duration: 0.12, ease: EASE_OUT } }}
21 whileTap={PRESS}
22 transition={{ duration: 0.18, ease: EASE_OUT }}
```

## src/components/layout/LyricsPanel.tsx

```text
59 initial={{ x: WIDTH }}
60 animate={{ x: 0 }}
61 exit={{ x: WIDTH, transition: RAIL_CLOSE }}
63 transition={RAIL_OPEN}
64 contain = "paint"
64 willChange = "transform"
66 fontSize = 14
66 fontWeight = 700
193 initial={{ x: WIDTH }}
194 animate={{ x: 0 }}
195 exit={{ x: WIDTH, transition: RAIL_CLOSE }}
197 transition={RAIL_OPEN}
205 boxShadow = "none"
206 contain = "paint"
207 willChange = "transform"
352 contentVisibility = "auto"
353 containIntrinsicSize = "0 40px"
428 fontSize = 11
429 fontWeight = 600
430 letterSpacing = "0.04em"
460 fontSize = 12.5
461 fontWeight = 600
463 letterSpacing = "0.004em"
485 whileHover={{ scale: 1.03 }}
486 whileTap={PRESS}
487 transition={{ duration: 0.12, ease: EASE_OUT }}
500 fontSize = 11.5
501 fontWeight = 650
502 letterSpacing = "0.004em"
503 boxShadow = "0 2px 8px rgba(0, 0, 0, 0.25)"
539 fontSize = 15
540 fontWeight = 700
546 fontSize = 12.5
569 fontSize = 12.5
570 fontWeight = 600
```

## src/components/layout/PlayerBar.tsx

```text
59 whileHover={disabled ? undefined : { scale: 1.1 }}
60 whileTap={disabled ? undefined : { scale: 0.92 }}
61 transition={BTN_SPRING}
104 whileHover={{ scale: 1.14 }}
105 whileTap={{ scale: 0.88 }}
106 transition={PLAY_SPRING}
119 initial={false}
122 initial={{ opacity: 0, scale: 0.5, filter: "blur(6px)" }}
122 filter = "blur(6px)"
123 animate={{ opacity: 1, scale: 1, filter: "blur(0px)", transition: { scale: { type: "spring", stiffness: 520, damping: 20 }, opacity: { duration: 0.16, ease: EASE_OUT }, filter: { duration: 0.2, ease: EASE_OUT }, }, }}
124 filter = "blur(0px)"
128 filter = { duration: 0.2, ease: EASE_OUT }
131 exit={{ opacity: 0, scale: 0.5, filter: "blur(6px)", transition: { duration: 0.14, ease: EASE_OUT } }}
131 filter = "blur(6px)"
316 initial={{ opacity: 0, scale: 0.88, y: 10, filter: "blur(6px)" }}
316 filter = "blur(6px)"
317 animate={{ opacity: 1, scale: 1, y: 0, filter: "blur(0px)", transition: { scale: { type: "spring", stiffness: 480, damping: 26 }, y: { type: "spring", stiffness: 480, damping: 26 }, opacity: { duration: 0.14, ease: EASE_OUT }, filter: { duration: 0.18, ease: EASE_OUT }, }, }}
318 filter = "blur(0px)"
323 filter = { duration: 0.18, ease: EASE_OUT }
327 exit={{ opacity: 0, scale: 0.94, y: 6, filter: "blur(4px)", transition: { duration: 0.12, ease: EASE_OUT } }}
327 filter = "blur(4px)"
342 backdropFilter = "blur(24px) saturate(1.4)"
343 WebkitBackdropFilter = "blur(24px) saturate(1.4)"
345 boxShadow = "0 16px 40px rgba(0, 0, 0, 0.55), inset 0 1px 0 rgba(255, 255, 255, 0.08)"
352 fontSize = 10
352 fontWeight = 600
352 lineHeight = 1
545 fontSize = 10.5
546 fontWeight = 500
547 lineHeight = 1
618 boxShadow = isLineExpanded ? "0 0 8px rgba(255, 255, 255, 0.45)" : "none"
627 initial={{ opacity: 0, y: 3, scale: 0.94 }}
628 animate={{ opacity: 1, y: 0, scale: 1 }}
629 exit={{ opacity: 0, y: 3, scale: 0.94 }}
630 transition={{ duration: 0.12, ease: EASE_OUT }}
648 boxShadow = "0 4px 14px rgba(0, 0, 0, 0.55)"
649 fontSize = 10.5
650 fontWeight = 600
651 letterSpacing = "var(--type-caption-track)"
875 initial={{ opacity: 0, y: 8, scale: 0.97 }}
876 animate={{ opacity: 1, y: 0, scale: 1 }}
877 exit={{ opacity: 0, y: 6, scale: 0.98, transition: { duration: 0.14, ease: EASE_OUT } }}
878 transition={{ duration: 0.22, ease: EASE_OUT }}
879 whileTap={PRESS}
890 backdropFilter = "blur(20px)"
891 WebkitBackdropFilter = "blur(20px)"
894 boxShadow = "0 8px 24px rgba(0, 0, 0, 0.4)"
901 fontSize = 11.5
902 fontWeight = 500
903 lineHeight = 1
910 fontWeight = 600
911 fontSize = 10.5
935 backdropFilter = immersive ? undefined : "blur(32px) saturate(160%)"
936 WebkitBackdropFilter = immersive ? undefined : "blur(32px) saturate(160%)"
941 boxShadow = immersive ? "inset 0 1px 0 0 rgba(255, 255, 255, 0.09), 0 20px 50px rgba(0, 0, 0, 0.30)" : "inset 0 1px 0 0 rgba(255, 255, 255, 0.1), 0 12px 32px rgba(0, 0, 0, 0.45)"
1038 boxShadow = "0 2px 8px rgba(0, 0, 0, 0.4)"
1049 initial={false}
1052 initial={{ opacity: 0, filter: "blur(10px)", scale: 1.06 }}
1052 filter = "blur(10px)"
1053 animate={{ opacity: 1, filter: "blur(0px)", scale: 1 }}
1053 filter = "blur(0px)"
1054 exit={{ opacity: 0, filter: "blur(8px)", scale: 1.04 }}
1054 filter = "blur(8px)"
1055 transition={{ duration: 0.3, ease: EASE_OUT }}
1085 boxShadow = "inset 0 0 0 1px rgba(255, 255, 255, 0.08)"
1100 fontSize = 14.5
1100 fontWeight = 600
1100 lineHeight = 1.25
1100 letterSpacing = "-0.01em"
1117 fontSize = 12.5
1117 fontWeight = 400
1117 lineHeight = 1.2
```

## src/components/layout/QueuePanel.tsx

```text
62 fontSize = 9
62 fontWeight = 700
113 fontSize = 12.5
113 fontWeight = 500
116 fontSize = 11
135 fontSize = 11
176 fontSize = 13
176 fontWeight = 600
179 fontSize = 11.5
191 fontSize = 10.5
191 fontWeight = 700
191 letterSpacing = "0.08em"
199 fontSize = 11
199 fontWeight = 600
211 fontSize = 12
346 initial={{ x: WIDTH }}
347 animate={{ x: 0 }}
348 exit={{ x: WIDTH, transition: RAIL_CLOSE }}
349 transition={RAIL_OPEN}
354 boxShadow = "none"
355 willChange = "transform"
361 fontSize = 14
361 fontWeight = 700
361 letterSpacing = "-0.01em"
385 fontSize = 12
402 initial={false}
408 initial={{ opacity: 0, height: 0 }}
409 animate={{ opacity: 1, height: "auto" }}
410 exit={{ opacity: 0, height: 0, transition: { duration: 0.18, ease: EASE_DRAWER } }}
411 transition={{ duration: 0.2, ease: EASE_OUT }}
414 boxShadow = "0 10px 26px rgba(0,0,0,0.45)"
```

## src/components/layout/Sidebar.tsx

```text
55 whileTap={{ scale: 0.98 }}
56 transition={PRESS_TRANSITION}
73 fontSize = 13.5
74 fontWeight = active ? 600 : 500
84 transition={SPRING}
90 boxShadow = "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))"
187 initial={false}
188 animate={{ opacity: visible ? 1 : 0 }}
189 transition={{ duration: 0.15, ease: EASE_OUT }}
201 boxShadow = "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))"
256 fontSize = 11
256 fontWeight = 600
257 letterSpacing = "0.06em"
263 animate={{ rotate: expanded ? 0 : -90 }}
264 transition={{ duration: 0.18, ease: EASE_OUT }}
270 initial={false}
273 initial={{ height: 0, opacity: 0 }}
274 animate={{ height: "auto", opacity: 1 }}
275 exit={{ height: 0, opacity: 0 }}
276 transition={{ duration: 0.2, ease: EASE_OUT }}
427 animate={{ width: railWidth }}
428 transition={{ duration: 0.22, ease: EASE_OUT }}
461 boxShadow = "inset 0 0 0 1px rgba(0, 0, 0, 0.18)"
505 fontSize = 12
507 lineHeight = 1.5
547 fontSize = 13
547 fontWeight = 600
555 fontSize = 11
555 fontWeight = 600
579 whileTap={{ scale: 0.98 }}
580 transition={PRESS_TRANSITION}
609 transition={SPRING}
615 boxShadow = "0 2px 10px var(--color-accent-dim, rgba(88, 115, 216, 0.35))"
622 boxShadow = isCollapsed ? "0 2px 6px rgba(0, 0, 0, 0.35)" : undefined
630 fontSize = 13
630 fontWeight = active ? 600 : 500
```

## src/components/layout/TitleBar.tsx

```text
165 fontSize = 13
165 fontWeight = 500
270 boxShadow = "0 0 0 1.5px rgba(16, 16, 20, 0.95)"
282 initial={{ opacity: 0, y: -6, scale: 0.97 }}
283 animate={{ opacity: 1, y: 0, scale: 1 }}
285 exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.12, ease: EASE_OUT } }}
286 transition={{ duration: 0.16, ease: EASE_OUT }}
296 backdropFilter = "blur(40px) saturate(1.4)"
297 WebkitBackdropFilter = "blur(40px) saturate(1.4)"
299 boxShadow = "0 16px 40px rgba(0,0,0,0.5)"
313 fontSize = 13.5
313 fontWeight = 600
318 fontSize = 11.5
```

## src/components/layout/TopSearch.tsx

```text
23 fontSize = 10.5
24 fontWeight = 700
25 letterSpacing = "0.08em"
33 fontSize = 13
33 fontWeight = 600
39 fontSize = 11.5
40 letterSpacing = "var(--type-caption-track)"
45 fontSize = 10
45 fontWeight = 650
46 letterSpacing = "0.01em"
254 initial={instantOpenRef.current ? false : { opacity: 0, y: -4, scale: 0.98 }}
255 animate={{ opacity: 1, y: 0, scale: 1 }}
256 exit={{ opacity: 0, y: -4, scale: 0.98, transition: { duration: 0.1, ease: EASE_OUT } }}
257 transition={{ duration: 0.14, ease: EASE_OUT }}
269 backdropFilter = "blur(40px) saturate(1.4)"
270 WebkitBackdropFilter = "blur(40px) saturate(1.4)"
272 boxShadow = "0 16px 40px rgba(0, 0, 0, 0.5)"
310 fontSize = 13.5
310 fontWeight = 600
317 fontSize = 12
320 fontSize = 12
342 fontSize = 11
```

## src/components/playground/AnimatedIcons.tsx

```text
104 initial={false}
105 animate={liked ? { scale: [1, 1.24, 0.96, 1] } : { scale: 1 }}
106 transition={{ duration: 0.3, ease: [0.23, 1, 0.32, 1] }}
```

## src/components/playground/PlaygroundCards.tsx

```text
24 whileHover={{ y: -3 }}
25 transition={{ type: "spring", stiffness: 520, damping: 44 }}
66 fontSize = 14
67 fontWeight = 500
68 lineHeight = "18px"
85 fontSize = 12
86 lineHeight = "15px"
191 whileHover={{ y: -3 }}
192 transition={{ type: "spring", stiffness: 520, damping: 44 }}
221 backdropFilter = "blur(20px)"
222 WebkitBackdropFilter = "blur(20px)"
256 animate={{ x: currentTransform.x, y: currentTransform.y, rotate: currentTransform.rotate, scale: currentTransform.scale, }}
262 transition={{ type: "spring", stiffness: 380, damping: 26 }}
273 boxShadow = "0 6px 18px rgba(0, 0, 0, 0.4)"
301 backdropFilter = "blur(24px) saturate(180%)"
302 WebkitBackdropFilter = "blur(24px) saturate(180%)"
313 boxShadow = "0 -4px 16px rgba(0, 0, 0, 0.2)"
318 fontSize = 14
319 fontWeight = 700
320 letterSpacing = "-0.01em"
```

## src/components/playground/PlaygroundControls.tsx

```text
19 fontSize = 13
20 fontSize = 14
21 fontSize = 15
26 whileHover={{ scale: 1.04 }}
27 whileTap={{ scale: 0.96 }}
41 fontSize = sizeStyles.fontSize
42 fontWeight = 700
43 letterSpacing = "-0.01em"
45 boxShadow = playing ? "0 0 0 4px var(--color-accent-dim), 0 4px 18px var(--color-accent-dim)" : "0 4px 18px -2px var(--color-accent-dim)"
74 initial={{ scale: 0.85, opacity: 0.7 }}
75 animate={{ scale: 1.5, opacity: 0 }}
76 exit={{ opacity: 0 }}
77 transition={{ repeat: Infinity, duration: 1.8, ease: "easeOut" }}
90 whileHover={{ scale: 1.06 }}
91 whileTap={{ scale: 0.94 }}
107 boxShadow = "0 8px 24px -4px var(--color-accent-dim)"
265 fontSize = "0.875rem"
266 fontWeight = 600
279 transition={{ type: "spring", stiffness: 420, damping: 38, mass: 0.8, }}
291 boxShadow = "0 4px 14px -2px var(--color-accent-dim)"
```

## src/components/playground/PlaygroundDials.tsx

```text
34 fontSize = 13
34 fontWeight = 600
52 layout
53 transition={{ type: "spring", stiffness: 500, damping: 30 }}
59 boxShadow = "0 2px 5px rgba(0,0,0,0.3)"
103 whileHover={{ scale: 1.10 }}
104 whileTap={{ scale: 0.88 }}
105 transition={{ type: "spring", stiffness: 450, damping: 22 }}
142 fontSize = 11
```

## src/components/playground/PlaygroundThemeBar.tsx

```text
40 backdropFilter = "blur(20px)"
58 boxShadow = "0 0 20px -4px var(--color-accent-dim)"
64 fontSize = 14
64 fontWeight = 700
64 letterSpacing = "-0.01em"
67 fontSize = 12
79 whileHover={{ scale: 1.04 }}
80 whileTap={{ scale: 0.96 }}
101 boxShadow = isSelected ? `0 0 12px ${preset.hex}` : "none"
111 fontSize = 12
112 fontWeight = isSelected ? 600 : 500
135 fontSize = 12
```

## src/components/playground/PlaygroundTracks.tsx

```text
53 whileHover={{ scale: 1.15 }}
54 whileTap={{ scale: 0.88 }}
76 fontSize = 13
88 fontSize = 14
89 fontWeight = 500
105 animate={{ height: ["20%", "100%", "20%"] }}
106 transition={{ repeat: Infinity, duration: 0.55 + i * 0.15, ease: "easeInOut" }}
114 fontSize = 12.5
162 fontSize = 12.5
```

## src/components/podcast/PodcastStage.tsx

```text
53 initial={false}
66 initial={{ opacity: 0, scale: 0.98 }}
67 animate={{ opacity: 1, scale: 1 }}
68 exit={{ opacity: 0 }}
69 transition={{ duration: 0.4, ease: EASE_OUT }}
70 boxShadow = "0 36px 90px rgba(0,0,0,0.55)"
77 initial={{ opacity: 0, scale: 0.98 }}
78 animate={{ opacity: 1, scale: 1 }}
79 exit={{ opacity: 0 }}
80 transition={{ duration: 0.4, ease: EASE_OUT }}
81 boxShadow = "0 36px 90px rgba(0,0,0,0.55)"
86 fontSize = "clamp(12px, 1vw, 14px)"
86 fontWeight = 700
86 letterSpacing = "0.08em"
89 fontSize = "clamp(24px, 2.8vw, 40px)"
89 fontWeight = 800
89 letterSpacing = "-0.025em"
89 lineHeight = 1.1
104 fontSize = 12
104 fontWeight = 700
104 letterSpacing = "0.08em"
105 fontSize = 15
105 lineHeight = 1.65
```

## src/components/podcast/TranscriptView.tsx

```text
92 fontSize = big ? 13 : 11.5
92 fontWeight = 700
92 letterSpacing = "0.06em"
101 contentVisibility = "auto"
102 containIntrinsicSize = big ? "auto 160px" : "auto 110px"
110 boxShadow = `0 0 10px ${color}`
111 fontSize = big ? 13 : 11.5
111 fontWeight = 650
114 fontSize = big ? "clamp(21px, 1.9vw, 28px)" : "16px"
114 lineHeight = big ? 1.42 : 1.55
114 fontWeight = big ? 650 : 560
114 letterSpacing = "-0.012em"
135 textShadow = state === "now" ? `0 0 22px rgba(${ink}, 0.28)` : "none"
227 fontSize = 14
230 fontSize = 14
278 initial={{ opacity: 0, y: 10, x: "-50%" }}
279 animate={{ opacity: 1, y: 0, x: "-50%" }}
280 exit={{ opacity: 0, y: 10, x: "-50%", transition: { duration: 0.14 } }}
281 transition={{ duration: 0.2, ease: EASE_OUT }}
295 backdropFilter = "blur(14px)"
296 WebkitBackdropFilter = "blur(14px)"
298 fontSize = 12.5
299 fontWeight = 650
301 boxShadow = "0 10px 28px rgba(0,0,0,0.35)"
```

## src/components/ui/AddToPlaylistModal.tsx

```text
72 fontSize = 15
72 fontWeight = 700
105 fontSize = 13.5
114 fontSize = 13.5
114 fontWeight = 600
130 fontSize = 13.5
130 fontWeight = 600
131 fontSize = 11.5
150 fontSize = 12.5
```

## src/components/ui/AlbumCard.tsx

```text
35 layout="position"
36 transition={{ layout: REFLOW_SPRING }}
91 layout="position"
98 whileHover={{ y: -3 }}
99 whileTap={{ scale: 0.98 }}
100 transition={{ type: "spring", stiffness: 480, damping: 36, ...getGridItemTransition(index), }}
133 boxShadow = hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)"
156 fontSize = 13.5
157 fontWeight = 600
158 letterSpacing = "-0.012em"
159 lineHeight = "17px"
177 lineHeight = "15px"
191 fontSize = 12
192 lineHeight = "15px"
```

## src/components/ui/ArtistAbout.tsx

```text
43 whileTap={PRESS}
44 transition={PRESS_TRANSITION}
46 fontSize = 12.5
46 fontWeight = 600
63 fontSize = 13
64 fontWeight = 600
67 fontWeight = 500
83 fontSize = 22
83 fontWeight = 800
83 letterSpacing = "-0.02em"
84 fontSize = 12
96 initial={false}
102 initial={{ opacity: 0 }}
103 animate={{ opacity: 1 }}
104 exit={{ opacity: 0 }}
105 transition={{ duration: 0.3, ease: EASE_OUT }}
120 fontSize = 30
120 fontWeight = 800
120 letterSpacing = "-0.02em"
133 fontSize = 14.5
133 lineHeight = 1.65
136 fontSize = 11
136 fontWeight = 700
136 letterSpacing = "0.08em"
164 whileTap={{ scale: 0.99 }}
184 animate={{ scale: hover ? 1.03 : 1 }}
185 transition={{ duration: 0.5, ease: EASE_OUT }}
193 boxShadow = "0 8px 24px rgba(0,0,0,0.35)"
194 fontSize = 20
194 fontWeight = 800
194 lineHeight = 1
195 fontSize = 10
195 fontWeight = 600
201 fontSize = 28
201 fontWeight = 800
201 letterSpacing = "-0.02em"
202 fontSize = 14
202 fontWeight = 600
206 fontSize = 14
206 lineHeight = 1.5
210 fontSize = 13
210 fontWeight = 700
219 fontSize = 11
219 fontWeight = 700
219 letterSpacing = "0.08em"
```

## src/components/ui/ArtistCard.tsx

```text
28 layout="position"
29 transition={{ layout: REFLOW_SPRING }}
47 layout="position"
54 whileHover={{ y: -3 }}
55 whileTap={{ scale: 0.98 }}
56 transition={{ type: "spring", stiffness: 520, damping: 44, ...getGridItemTransition(index), }}
91 boxShadow = hover ? "0 14px 32px -4px rgba(0, 0, 0, 0.55), 0 0 20px var(--color-accent-dim)" : "0 6px 18px rgba(0, 0, 0, 0.35)"
99 animate={{ scale: hover ? 1.05 : 1 }}
100 transition={{ duration: 0.35, ease: EASE_OUT }}
120 fontSize = 32
121 fontWeight = 700
122 letterSpacing = "-0.02em"
155 initial={{ opacity: 0, scale: 0.9, y: 6 }}
156 animate={{ opacity: 1, scale: 1, y: 0 }}
157 exit={{ opacity: 0, scale: 0.9, y: 6, transition: { duration: 0.12, ease: EASE_OUT } }}
159 transition={SPRING}
172 boxShadow = "0 6px 16px rgba(0, 0, 0, 0.45)"
186 fontSize = 14
187 fontWeight = 600
188 lineHeight = "17px"
205 fontSize = 12
```

## src/components/ui/ArtistTour.tsx

```text
38 fontSize = 10.5
38 fontWeight = 800
38 letterSpacing = "0.06em"
39 fontSize = 21
39 fontWeight = 800
42 fontSize = 14
42 fontWeight = 650
43 fontSize = 12.5
44 fontSize = 11.5
63 whileHover={m.url ? { y: -3 } : undefined}
64 whileTap={m.url ? PRESS : undefined}
65 transition={PRESS_TRANSITION}
68 boxShadow = "0 4px 14px rgba(0,0,0,0.3)"
76 fontSize = 13.5
76 fontWeight = 600
77 fontSize = 12
```

## src/components/ui/CirclePlayButton.tsx

```text
31 initial={{ opacity: 0, scale: 0.9 }}
32 animate={visible ? { opacity: 1, scale: 1 } : { opacity: 0, scale: 0.9 }}
33 whileHover={{ scale: 1.03 }}
34 whileTap={PRESS}
35 transition={{ duration: 0.16, ease: EASE_OUT }}
49 boxShadow = "0 4px 14px rgba(0,0,0,0.5)"
```

## src/components/ui/ConfirmDialog.tsx

```text
34 fontSize = 16
34 fontWeight = 700
37 fontSize = 13.5
37 lineHeight = 1.5
```

## src/components/ui/ContextMenu.tsx

```text
106 initial={animated ? { opacity: 0, scale: 0.96 } : false}
107 animate={{ opacity: 1, scale: 1 }}
108 transition={{ duration: 0.15, ease: EASE_OUT }}
119 backdropFilter = "blur(20px)"
120 WebkitBackdropFilter = "blur(20px)"
122 boxShadow = "0 12px 32px rgba(0,0,0,0.5)"
146 fontSize = 13
147 fontWeight = 500
```

## src/components/ui/CoverArt.tsx

```text
61 fontSize = size * 0.4
61 fontWeight = 700
```

## src/components/ui/CreditsList.tsx

```text
25 fontSize = size * 0.38
26 fontWeight = 700
28 textShadow = "0 1px 2px rgba(0,0,0,0.35)"
80 fontSize = 11
80 fontWeight = 700
80 letterSpacing = "0.08em"
87 fontSize = 14
87 fontWeight = 650
91 fontSize = 11
91 fontWeight = 600
91 lineHeight = "18px"
114 fontSize = 11.5
```

## src/components/ui/CreditsModal.tsx

```text
20 boxShadow = "0 8px 22px rgba(0,0,0,0.4)"
24 fontSize = 11
24 fontWeight = 700
24 letterSpacing = "0.08em"
25 fontSize = 16
25 fontWeight = 700
28 fontSize = 12.5
```

## src/components/ui/Dropdown.tsx

```text
57 whileTap={PRESS}
58 transition={PRESS_TRANSITION}
66 fontSize = 12.5
66 fontWeight = 500
72 animate={{ rotate: open ? 180 : 0 }}
72 transition={{ duration: 0.18, ease: EASE_OUT }}
83 initial={{ opacity: 0, scale: 0.95, y: -4 }}
84 animate={{ opacity: 1, scale: 1, y: 0 }}
86 exit={{ opacity: 0, scale: 0.96, y: -4, transition: { duration: 0.1, ease: EASE_OUT } }}
87 transition={{ duration: 0.14, ease: EASE_OUT }}
95 backdropFilter = "blur(20px)"
95 WebkitBackdropFilter = "blur(20px)"
97 boxShadow = "0 12px 32px rgba(0,0,0,0.45)"
113 fontSize = 12.5
113 fontWeight = active ? 600 : 500
```

## src/components/ui/EditPlaylistModal.tsx

```text
48 fontSize = 12
48 fontWeight = 600
66 fontSize = 14
150 fontSize = 17
150 fontWeight = 700
150 letterSpacing = "-0.01em"
167 boxShadow = "0 14px 34px rgba(0,0,0,0.45)"
182 fontSize = 15
182 fontWeight = 600
193 lineHeight = 1.45
201 fontSize = 12
201 fontWeight = 600
236 fontSize = 11.5
```

## src/components/ui/EmptyState.tsx

```text
24 initial={{ opacity: 0, y: 10 }}
25 animate={{ opacity: 1, y: 0 }}
26 transition={{ duration: 0.25, ease: [0.23, 1, 0.32, 1] }}
58 fontSize = 17
59 fontWeight = 700
60 letterSpacing = "-0.01em"
70 fontSize = 13.5
71 lineHeight = 1.5
85 fontSize = 12
```

## src/components/ui/EpisodeRow.tsx

```text
100 whileTap={PRESS}
101 transition={PRESS_TRANSITION}
113 fontSize = 13
120 boxShadow = "0 2px 6px rgba(0,0,0,0.25)"
128 fontSize = 13.5
128 fontWeight = 600
128 letterSpacing = "-0.012em"
131 fontSize = 12
133 fontSize = 9.5
133 fontWeight = 700
164 fontSize = 12.5
```

## src/components/ui/EvenGrid.tsx

```text
183 layout="position"
184 transition={{ layout: REFLOW_SPRING }}
198 layout="position"
200 transition={getGridItemTransition(index)}
```

## src/components/ui/ExpandableDescription.tsx

```text
31 fontSize = 13.5
32 lineHeight = 1.5
59 fontSize = 12
60 fontWeight = 600
```

## src/components/ui/Loader.tsx

```text
51 fontSize = 12.5
51 fontWeight = 500
51 letterSpacing = "0.01em"
```

## src/components/ui/MediaTile.tsx

```text
37 layout="position"
44 whileHover={{ y: -3 }}
45 whileTap={{ scale: 0.98 }}
46 transition={{ type: "spring", stiffness: 480, damping: 36, ...getGridItemTransition(index) }}
71 boxShadow = hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)"
80 fontSize = 13.5
81 fontWeight = 600
82 letterSpacing = "-0.012em"
83 lineHeight = "17px"
99 fontSize = 12
100 lineHeight = "15px"
```

## src/components/ui/Modal.tsx

```text
34 initial={{ opacity: 0 }}
35 animate={{ opacity: 1, transition: { duration: 0.16 } }}
36 exit={{ opacity: 0, transition: { duration: 0.12 } }}
41 backdropFilter = "blur(2px)"
51 initial={{ opacity: 0, scale: 0.96, y: 8 }}
52 animate={{ opacity: 1, scale: 1, y: 0, transition: { duration: 0.2, ease: EASE_OUT } }}
53 exit={{ opacity: 0, scale: 0.97, y: 6, transition: { duration: 0.12, ease: EASE_OUT } }}
60 boxShadow = "0 24px 64px rgba(0,0,0,0.6)"
```

## src/components/ui/PageHeader.tsx

```text
59 boxShadow = "0 20px 54px rgba(0, 0, 0, 0.6)"
83 fontSize = 11
83 letterSpacing = "0.06em"
92 fontSize = "clamp(24px, 3.8vw, 40px)"
93 lineHeight = 1.06
94 letterSpacing = "-0.028em"
```

## src/components/ui/PlayActions.tsx

```text
172 layout
173 initial={false}
176 whileTap={empty ? undefined : PRESS}
177 transition={{ layout: REFLOW_SPRING, default: PRESS_TRANSITION }}
192 fontSize = 13.5
193 fontWeight = 700
194 letterSpacing = "-0.01em"
197 boxShadow = playing ? "0 0 0 4px rgba(255, 255, 255, 0.25), 0 4px 16px rgba(0, 0, 0, 0.35)" : "0 2px 10px rgba(255, 255, 255, 0.20)"
204 layout="position"
204 transition={{ layout: REFLOW_SPRING }}
212 initial={false}
216 layout="position"
217 initial={{ opacity: 0 }}
218 animate={{ opacity: 1 }}
219 exit={{ opacity: 0 }}
220 transition={{ opacity: { duration: 0.15, ease: EASE_OUT }, layout: REFLOW_SPRING }}
232 layout
233 initial={false}
236 whileTap={empty ? undefined : PRESS}
237 transition={{ layout: REFLOW_SPRING, default: PRESS_TRANSITION }}
256 fontSize = 13.5
257 fontWeight = 600
258 letterSpacing = "-0.01em"
267 layout="position"
268 animate={{ rotate: shuffleActive ? [0, -15, 15, 0] : 0, }}
271 transition={{ rotate: { duration: 0.35, ease: EASE_OUT }, layout: REFLOW_SPRING }}
282 initial={false}
286 layout="position"
287 initial={{ opacity: 0 }}
288 animate={{ opacity: 1 }}
289 exit={{ opacity: 0 }}
290 transition={{ opacity: { duration: 0.15, ease: EASE_OUT }, layout: REFLOW_SPRING }}
313 whileTap={PRESS}
314 transition={PRESS_TRANSITION}
320 fontSize = 13
321 fontWeight = 600
342 whileTap={PRESS}
343 transition={PRESS_TRANSITION}
350 fontSize = 13
351 fontWeight = 600
```

## src/components/ui/QuitConfirm.tsx

```text
24 fontSize = 16
24 fontWeight = 700
27 fontSize = 13
27 lineHeight = 1.55
```

## src/components/ui/ReleaseCountdown.tsx

```text
25 lineHeight = "15px"
34 fontWeight = 600
```

## src/components/ui/SaveAlbumButton.tsx

```text
27 whileTap={PRESS}
28 transition={PRESS_TRANSITION}
34 fontSize = 13
35 fontWeight = 600
```

## src/components/ui/SectionTitle.tsx

```text
37 fontSize = "clamp(1.286rem, 2.2vw, var(--type-section-size))"
57 fontSize = 12.5
57 fontWeight = 600
```

## src/components/ui/SortDirectionToggle.tsx

```text
26 whileTap={PRESS}
27 transition={PRESS_TRANSITION}
37 initial={false}
38 animate={{ rotate: asc ? 180 : 0 }}
39 transition={{ duration: 0.2, ease: EASE_OUT }}
```

## src/components/ui/SortToolbar.tsx

```text
62 fontSize = 13
62 fontWeight = 400
```

## src/components/ui/SpeedControl.tsx

```text
44 whileTap={PRESS}
45 transition={PRESS_TRANSITION}
56 fontSize = 12.5
57 fontWeight = 700
67 initial={{ opacity: 0, y: up ? 6 : -6, scale: 0.97 }}
68 animate={{ opacity: 1, y: 0, scale: 1 }}
69 exit={{ opacity: 0, y: up ? 4 : -4, scale: 0.98, transition: { duration: 0.1 } }}
70 transition={{ duration: 0.16, ease: EASE_OUT }}
83 boxShadow = "0 16px 40px rgba(0,0,0,0.5)"
87 fontSize = 10.5
87 fontWeight = 700
87 letterSpacing = "0.08em"
110 fontSize = 13
111 fontWeight = v === speed ? 700 : 500
```

## src/components/ui/Switch.tsx

```text
39 layout
40 transition={{ ...SPRING, duration: 0.25 }}
46 boxShadow = "0 2px 5px rgba(0,0,0,0.3)"
```

## src/components/ui/SyncButton.tsx

```text
42 fontSize = 12
55 fontSize = 12
55 fontWeight = 500
```

## src/components/ui/Toaster.tsx

```text
24 initial={false}
30 layout
33 initial={{ opacity: 0, y: 14, scale: 0.96 }}
34 animate={{ opacity: 1, y: 0, scale: 1 }}
36 exit={{ opacity: 0, y: 8, scale: 0.97, transition: { duration: 0.15, ease: [0.23, 1, 0.32, 1] } }}
37 transition={{ type: "spring", bounce: 0, duration: 0.35 }}
42 backdropFilter = "blur(20px)"
42 WebkitBackdropFilter = "blur(20px)"
44 boxShadow = "0 12px 36px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.05)"
45 fontSize = 13
45 fontWeight = 600
46 letterSpacing = "-0.006em"
63 fontSize = 12.5
```

## src/components/ui/Tooltip.tsx

```text
190 initial={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
191 animate={isRight ? { opacity: 1, x: 0, y: "-50%", scale: 1 } : { opacity: 1, y: 0, scale: 1, x: tx }}
192 exit={isRight ? { opacity: 0, x: -4, y: "-50%", scale: 0.94 } : { opacity: 0, y: off, scale: 0.94, x: tx }}
193 transition={instant ? { duration: 0 } : { duration: 0.15, ease: EASE_OUT }}
204 lineHeight = 1.2
206 backdropFilter = "blur(20px)"
207 WebkitBackdropFilter = "blur(20px)"
209 boxShadow = "0 8px 22px rgba(0,0,0,0.45)"
210 fontSize = 11.5
211 fontWeight = 600
```

## src/components/ui/TrackRow.tsx

```text
65 whileTap={PRESS}
66 transition={PRESS_TRANSITION}
202 whileTap={PRESS}
203 transition={PRESS_TRANSITION}
235 fontSize = 13
247 boxShadow = "0 2px 6px rgba(0,0,0,0.25)"
261 fontSize = 13.5
262 fontWeight = 600
263 letterSpacing = "-0.012em"
291 fontSize = 12
293 fontSize = 9.5
293 fontWeight = 700
350 initial={false}
354 initial={{ scale: 0.8, opacity: 0 }}
355 animate={{ scale: 1, opacity: 1 }}
356 exit={{ scale: 0.8, opacity: 0 }}
357 transition={{ duration: 0.14, ease: EASE_OUT }}
365 initial={{ scale: 0.8, opacity: 0 }}
366 animate={{ scale: 1, opacity: 1 }}
367 exit={{ scale: 0.8, opacity: 0 }}
368 transition={{ duration: 0.14, ease: EASE_OUT }}
381 fontSize = 12.5
```

## src/components/ui/TrackToolbar.tsx

```text
94 fontSize = 13
94 fontWeight = 400
```

## src/components/ui/YtMatchModal.tsx

```text
94 fontSize = 15
94 fontWeight = 700
97 fontSize = 12
109 fontSize = 12
109 lineHeight = 1.45
113 fontWeight = 600
128 fontSize = 11.5
158 fontSize = 13.5
158 fontWeight = 600
162 fontSize = 11.5
201 fontSize = 9
201 fontWeight = 700
205 fontSize = 12.5
208 fontSize = 12.5
```

## src/lib/lyrics.tsx

```text
232 textShadow = "none"
236 textShadow = `0 0 ${tone.blur.toFixed(1)}px rgba(${ink}, 0.95), ` + `0 0 ${(tone.blur * 0.42).toFixed(1)}px rgba(${ink}, 0.82)`
566 fontSize = size
567 lineHeight = 1.26
568 letterSpacing = tracking
569 fontWeight = weight
583 backgroundImage = `linear-gradient(90deg, rgba(${inkRgb}, 0.99) var(--a,-50%), rgba(${inkRgb}, ${dim}) var(--b,-10%))`
```

## src/lib/mesh.ts

```text
43 backgroundImage = `url(/basegradient${imgIdx}.png), radial-gradient(circle at 0% 0%, ${a} 0%, transparent 70%), radial-gradient(circle at 100% 100%, ${b} 0%, transparent 70%), radial-gradient(circle at 50% 50%, ${c} 0%, transparent 70%), linear-gradient(140deg, ${base}, ${base})`
```

## src/pages/AlbumPage.tsx

```text
99 fontSize = 14.5
108 fontWeight = 700
120 fontSize = 10
121 fontWeight = 500
137 layout="position"
138 transition={{ layout: REFLOW }}
144 layout="position"
145 transition={{ layout: REFLOW }}
147 whileHover={{ zIndex: 40 }}
169 fontSize = 12
170 fontWeight = 500
177 fontSize = 11
```

## src/pages/ArtistPage.tsx

```text
125 whileTap={PRESS}
126 transition={PRESS_TRANSITION}
132 fontSize = 13
133 fontWeight = 600
151 fontSize = 14
151 fontWeight = 600
163 fontSize = 14.5
164 fontWeight = 500
172 fontSize = 10
192 whileTap={PRESS}
193 transition={PRESS_TRANSITION}
224 layout="position"
225 initial={{ opacity: 0, y: 8 }}
226 animate={{ opacity: 1, y: 0 }}
227 transition={{ layout: REFLOW_SPRING, duration: 0.28, delay: Math.min(i % TOP_TRACKS_COLLAPSED, 6) * 0.03, ease: EASE_OUT, }}
234 whileHover={{ zIndex: 40 }}
```

## src/pages/Home.tsx

```text
248 layout="position"
250 initial={animateIn ? { opacity: 0, y: 10 } : false}
251 animate={{ opacity: 1, y: 0 }}
252 whileTap={PRESS}
253 transition={{ opacity: { duration: 0.28, delay: index * 0.035, ease: EASE_OUT }, y: { duration: 0.28, delay: index * 0.035, ease: EASE_OUT }, scale: { duration: 0.12, ease: EASE_OUT }, layout: REFLOW, }}
278 boxShadow = lit ? "0 10px 24px rgba(0,0,0,0.42)" : "0 2px 6px rgba(0,0,0,0.18)"
296 boxShadow = "0 2px 8px rgba(69, 10, 245, 0.35)"
312 boxShadow = "0 2px 8px rgba(0,0,0,0.3)"
327 boxShadow = "0 2px 8px rgba(0,0,0,0.2)"
345 fontSize = 14
346 fontWeight = 700
347 letterSpacing = "-0.01em"
659 layout="position"
661 transition={{ layout: REFLOW }}
669 contain = "layout style"
703 layout="position"
705 transition={{ layout: REFLOW }}
709 whileTap={{ scale: 0.97 }}
736 boxShadow = hover ? "0 12px 28px rgba(0, 0, 0, 0.5)" : "0 4px 14px rgba(0, 0, 0, 0.3)"
772 fontSize = 13.5
773 fontWeight = 600
774 letterSpacing = "-0.012em"
775 lineHeight = "17px"
805 fontSize = 12
805 lineHeight = "15px"
858 layout="position"
858 transition={{ layout: REFLOW }}
895 layout="position"
895 transition={{ layout: REFLOW }}
948 fontSize = 12.5
988 layout="position"
988 transition={{ layout: REFLOW }}
995 fontWeight = 600
1016 layout="position"
1016 transition={{ layout: REFLOW }}
1023 fontWeight = 600
1045 layout="position"
1045 transition={{ layout: REFLOW }}
1075 fontSize = "clamp(2rem, 3.8vw, 2.714rem)"
1076 fontWeight = 400
1077 letterSpacing = "var(--type-display-track)"
1078 lineHeight = 1.14
1111 layout="position"
1113 transition={{ layout: REFLOW }}
1122 fontWeight = 800
```

## src/pages/Library.tsx

```text
76 layout="position"
76 transition={{ layout: REFLOW }}
78 layout="position"
79 transition={{ layout: REFLOW }}
81 fontSize = 12
81 letterSpacing = "0.06em"
81 fontWeight = 600
85 layout="position"
85 transition={{ layout: REFLOW }}
131 layout="position"
131 transition={{ layout: REFLOW }}
133 layout="position"
134 transition={{ layout: REFLOW }}
136 fontSize = 12
136 letterSpacing = "0.06em"
136 fontWeight = 600
140 layout="position"
140 transition={{ layout: REFLOW }}
175 layout="position"
175 transition={{ layout: REFLOW }}
177 layout="position"
178 transition={{ layout: REFLOW }}
180 fontSize = 12
180 letterSpacing = "0.06em"
180 fontWeight = 600
184 layout="position"
184 transition={{ layout: REFLOW }}
244 layout="position"
245 transition={{ layout: REFLOW }}
249 layout="position"
250 transition={{ layout: REFLOW }}
263 layout="position"
264 initial={{ opacity: 0 }}
265 animate={{ opacity: 1 }}
266 transition={{ opacity: { duration: 0.15, ease: [0.23, 1, 0.32, 1] }, layout: REFLOW, }}
```

## src/pages/Playground.tsx

```text
43 fontSize = 12
43 fontWeight = 700
47 fontSize = "clamp(24px, 3.2vw, 34px)"
47 fontWeight = 800
47 letterSpacing = "-0.03em"
50 fontSize = "clamp(13px, 1.4vw, 14.5px)"
50 lineHeight = 1.6
56 layout="position"
56 transition={{ layout: REFLOW }}
62 layout="position"
62 transition={{ layout: REFLOW }}
64 fontSize = "clamp(15px, 1.8vw, 18px)"
64 fontWeight = 700
64 letterSpacing = "-0.01em"
67 fontSize = "clamp(12px, 1.2vw, 13px)"
93 layout="position"
93 transition={{ layout: REFLOW }}
95 fontSize = "clamp(15px, 1.8vw, 18px)"
95 fontWeight = 700
95 letterSpacing = "-0.01em"
98 fontSize = "clamp(12px, 1.2vw, 13px)"
111 layout="position"
111 transition={{ layout: REFLOW }}
113 fontSize = "clamp(15px, 1.8vw, 18px)"
113 fontWeight = 700
113 letterSpacing = "-0.01em"
116 fontSize = "clamp(12px, 1.2vw, 13px)"
127 layout="position"
127 transition={{ layout: REFLOW }}
129 fontSize = "clamp(15px, 1.8vw, 18px)"
129 fontWeight = 700
129 letterSpacing = "-0.01em"
132 fontSize = "clamp(12px, 1.2vw, 13px)"
138 initial={true}
139 initial={false}
140 initial={75}
```

## src/pages/PlaylistPage.tsx

```text
203 fontWeight = 600
210 letterSpacing = "0.05em"
210 fontWeight = 600
229 whileTap={PRESS}
230 transition={PRESS_TRANSITION}
232 fontSize = 13
233 fontWeight = 600
266 layout="position"
267 transition={{ layout: REFLOW }}
```

## src/pages/Playlists.tsx

```text
64 layout="position"
65 transition={getGridItemTransition(index)}
69 whileHover={{ y: -3 }}
70 whileTap={{ scale: 0.98 }}
125 fontSize = "clamp(13px, 0.9vw, 14px)"
126 fontWeight = 600
131 lineHeight = "18px"
145 fontSize = 12
150 lineHeight = "15px"
189 layout="position"
190 transition={getGridItemTransition(index)}
193 whileHover={{ y: -3 }}
194 whileTap={{ scale: 0.98 }}
209 boxShadow = hover ? "0 12px 28px rgba(0,0,0,0.5)" : "0 4px 14px rgba(0,0,0,0.3)"
225 fontSize = 11.5
225 fontWeight = 700
226 backdropFilter = "blur(8px)"
226 WebkitBackdropFilter = "blur(8px)"
233 fontSize = "clamp(13px, 0.9vw, 14px)"
233 fontWeight = 600
233 lineHeight = "18px"
236 fontSize = 12
236 lineHeight = "15px"
322 layout="position"
323 layout="position"
380 layout="position"
381 transition={{ layout: REFLOW }}
```

## src/pages/ProfilePage.tsx

```text
171 whileTap={PRESS}
172 transition={PRESS_TRANSITION}
173 fontSize = 13
173 fontWeight = 600
184 whileTap={PRESS}
185 transition={PRESS_TRANSITION}
186 fontSize = 13
186 fontWeight = 600
```

## src/pages/Search.tsx

```text
169 letterSpacing = "inherit"
180 layout="position"
181 transition={{ layout: REFLOW_SPRING, scale: PRESS_TRANSITION }}
187 whileTap={{ scale: 0.99 }}
202 boxShadow = lit ? "0 14px 36px rgba(0, 0, 0, 0.4)" : "0 2px 10px rgba(0, 0, 0, 0.18)"
218 backgroundImage = `url(${coverUrl(image, 64) ?? image})`
221 filter = "blur(56px) saturate(1.25)"
255 boxShadow = "0 18px 40px rgba(0, 0, 0, 0.5)"
270 fontSize = 11
270 letterSpacing = "0.06em"
280 fontSize = "clamp(1.5rem, 2.6vw, 2.25rem)"
281 fontWeight = 900
282 lineHeight = 1.06
283 letterSpacing = "-0.028em"
313 fontSize = 14
313 fontWeight = 500
315 fontWeight = 700
317 fontSize = 10
330 whileTap={PRESS}
331 transition={PRESS_TRANSITION}
344 fontSize = 13.5
345 fontWeight = 700
346 letterSpacing = "-0.01em"
349 boxShadow = isPlaying ? "0 0 0 4px rgba(255, 255, 255, 0.25), 0 4px 16px rgba(0, 0, 0, 0.35)" : "0 2px 10px rgba(255, 255, 255, 0.20)"
399 layout="position"
400 transition={{ layout: REFLOW_SPRING }}
563 layout="position"
564 transition={{ layout: REFLOW_SPRING }}
577 fontSize = 11
577 letterSpacing = "0.06em"
586 fontSize = "clamp(24px, 3.8vw, 40px)"
587 lineHeight = 1.06
588 letterSpacing = "-0.028em"
642 initial={{ opacity: 0, y: 6 }}
643 animate={{ opacity: 1, y: 0 }}
644 transition={{ duration: 0.22, ease: EASE_OUT }}
651 layout="position"
652 transition={{ layout: REFLOW_SPRING }}
```

## src/pages/Settings.tsx

```text
75 fontSize = 12.5
91 fontSize = 14
109 fontSize = 12.5
109 fontWeight = 600
115 fontSize = 11.5
137 fontSize = 13.5
169 fontSize = 13.5
170 fontWeight = 600
191 whileHover={atDefault ? undefined : { scale: 1.03 }}
192 whileTap={atDefault ? undefined : { scale: 0.97 }}
193 transition={PRESS_TRANSITION}
245 fontSize = "1rem"
246 fontWeight = 600
256 fontSize = 12
257 lineHeight = 1.5
375 fontSize = 13
375 fontWeight = 700
623 fontSize = 12
624 fontWeight = 600
771 fontSize = 12.5
820 fontSize = 12
923 fontSize = 12.5
938 fontSize = 13
938 lineHeight = 1.5
965 fontSize = 13
966 lineHeight = 1.6
978 fontWeight = 500
1023 fontSize = 12.5
1034 fontSize = 12
1071 layout="position"
1072 transition={{ layout: REFLOW }}
1088 fontSize = "1.143rem"
1089 fontWeight = 600
1090 letterSpacing = "-0.014em"
1184 fontWeight = 700
1200 layout="position"
1201 transition={{ layout: REFLOW }}
1236 fontSize = "1.143rem"
1237 fontWeight = 600
1238 letterSpacing = "-0.014em"
1244 fontSize = 12
1254 animate={{ rotate: collapsed ? 0 : 180 }}
1255 transition={{ duration: 0.2, ease: EASE_OUT }}
1272 initial={false}
1276 initial={{ opacity: 0, height: 0 }}
1277 animate={{ opacity: 1, height: "auto" }}
1278 exit={{ opacity: 0, height: 0, transition: { duration: 0.18, ease: EASE_OUT } }}
1279 transition={{ duration: 0.26, ease: EASE_OUT }}
1291 fontSize = 13
1292 lineHeight = 1.6
1302 fontSize = 12
1380 fontSize = 12.5
```

## src/pages/ShowPage.tsx

```text
139 fontWeight = 600
141 letterSpacing = "0.05em"
141 fontWeight = 600
158 whileTap={PRESS}
159 transition={PRESS_TRANSITION}
160 fontSize = 13
160 fontWeight = 600
186 fontSize = 13
```

## src/pages/StatsPage.tsx

```text
38 fontSize = 12
38 fontWeight = 600
39 fontSize = "clamp(20px, 2.2vw, 28px)"
39 fontWeight = 800
39 letterSpacing = "-0.02em"
40 fontSize = 11.5
58 fontSize = 10
64 fontSize = 11
90 fontSize = 12.5
111 fontSize = 13
115 fontSize = 12.5
138 fontSize = 12.5
154 fontSize = 13
157 fontSize = 13.5
157 fontWeight = 600
158 fontSize = 12
162 fontSize = 12
261 fontSize = 12
```
