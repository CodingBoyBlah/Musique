import { describe, it, expect } from "vitest";
import { weldPunctuation } from "./lyrics";

/* Every token becomes an inline-block so the sweep gradient can clip to it, and
   a line may break between any two of them. weldPunctuation decides where those
   breaks are allowed to be.

   These assert the rules rather than one exact tokenisation, because the whole
   point is that the output does not depend on how the source split the line -
   Musixmatch emits punctuation standalone, hangs it off the previous word, and
   sometimes returns a whole phrase, and all three have to come out the same. */

const w = (text: string, startMs: number, endMs: number) => ({ text, startMs, endMs });
const texts = (ws: ReturnType<typeof weldPunctuation>) => ws.map((x) => x.text);

const CLOSING = /^[,.!?;:…)\]}»”’"'\-–—]/u;
const OPENING = /[([{«“‘¿¡]$/u;

function expectWellFormed(out: ReturnType<typeof weldPunctuation>) {
  out.forEach((tok, i) => {
    expect(tok.text.length, "no empty tokens").toBeGreaterThan(0);
    expect(tok.text.trim().length, "no whitespace-only tokens").toBeGreaterThan(0);
    // a break lands between tokens, so a token may not START with a mark that
    // has to stay attached to the word before it
    expect(CLOSING.test(tok.text), `token ${i} "${tok.text}" starts with closing punctuation`).toBe(false);
    // ...nor END with one that belongs to the word after it
    if (i < out.length - 1) {
      expect(OPENING.test(tok.text.trimEnd()), `token ${i} "${tok.text}" ends with opening punctuation`).toBe(false);
    }
    // internal whitespace would be a break opportunity inside one inline-block
    expect(/\S\s+\S/.test(tok.text), `token ${i} "${tok.text}" has internal whitespace`).toBe(false);
    expect(/\s{2,}/.test(tok.text), `token ${i} "${tok.text}" has a run of spaces`).toBe(false);
  });
}

function expectMonotonic(out: ReturnType<typeof weldPunctuation>) {
  out.forEach((tok, i) => {
    expect(tok.endMs).toBeGreaterThanOrEqual(tok.startMs);
    if (i) expect(tok.startMs).toBeGreaterThanOrEqual(out[i - 1].startMs);
  });
}

describe("weldPunctuation", () => {
  it("keeps a trailing comma on the word before it", () => {
    // the screenshot case: ", I lied to you" starting its own line
    const out = weldPunctuation([w("I lied to you", 0, 900), w(",", 900, 950), w(" I lied", 950, 1600)]);
    expectWellFormed(out);
    expect(texts(out)).toContain("you, ");
  });

  it("keeps an opening bracket on the word after it, however the source split it", () => {
    const standalone = weldPunctuation([w("again ", 0, 500), w("(", 500, 520), w("Baby,", 520, 900)]);
    const hungOffPrevious = weldPunctuation([w("again   (", 0, 520), w("Baby,", 520, 900)]);
    const spacedApart = weldPunctuation([w("again", 0, 500), w(" ( ", 500, 520), w("Baby,", 520, 900)]);

    for (const out of [standalone, hungOffPrevious, spacedApart]) {
      expectWellFormed(out);
      expect(texts(out)).toContain("(Baby,");
    }
  });

  it("collapses a run of spaces that would otherwise render as a mid-line gap", () => {
    const out = weldPunctuation([w("in my dream again   ", 0, 900), w("(Baby)", 900, 1200)]);
    expectWellFormed(out);
    expect(texts(out).join("")).toBe("in my dream again (Baby)");
  });

  it("preserves the line's words, apart from collapsed spacing", () => {
    const out = weldPunctuation([
      w("It was definitely a blessing", 0, 900),
      w(",", 900, 950),
      w(" wakin' beside you", 950, 1800),
    ]);
    expectWellFormed(out);
    expect(texts(out).join("")).toBe("It was definitely a blessing, wakin' beside you");
  });

  it("spreads a phrase's span over the words it splits into", () => {
    // a source that hands back a whole phrase must not leave every word after
    // the first with no duration to sweep across
    const out = weldPunctuation([w("Put myself to sleep", 0, 2000)]);
    expect(out.length).toBeGreaterThan(1);
    expectMonotonic(out);
    out.forEach((tok) => expect(tok.endMs).toBeGreaterThan(tok.startMs));
    expect(out[0].startMs).toBe(0);
    expect(out[out.length - 1].endMs).toBe(2000);
  });

  it("leaves a word-level source's timings alone", () => {
    const out = weldPunctuation([w("Put ", 0, 300), w("myself ", 300, 600), w("to ", 600, 900), w("sleep", 900, 1200)]);
    expect(texts(out)).toEqual(["Put ", "myself ", "to ", "sleep"]);
    expect(out.map((t) => [t.startMs, t.endMs])).toEqual([[0, 300], [300, 600], [600, 900], [900, 1200]]);
  });

  it("carries a welded mark's timing into the word it joins", () => {
    const out = weldPunctuation([w("you", 0, 900), w(",", 900, 1100)]);
    expect(out[0]).toMatchObject({ text: "you,", endMs: 1100 });
  });

  it("starts a bracketed word at the bracket", () => {
    const out = weldPunctuation([w("(", 500, 520), w("To", 520, 900)]);
    expect(out[0]).toMatchObject({ text: "(To", startMs: 500 });
  });

  it("forces timings monotonic even when a source overlaps them", () => {
    const out = weldPunctuation([w("one ", 500, 900), w("two ", 200, 400), w("three", 800, 600)]);
    expectMonotonic(out);
  });

  it("survives lines that are empty, blank, or nothing but punctuation", () => {
    expect(weldPunctuation([])).toEqual([]);
    expect(weldPunctuation([w("   ", 0, 100)])).toEqual([]);
    expect(texts(weldPunctuation([w("...", 0, 400)]))).toEqual(["..."]);
    expect(texts(weldPunctuation([w("(", 0, 100)]))).toEqual(["("]);
  });

  it("drops empty tokens instead of rendering them", () => {
    const out = weldPunctuation([w("", 0, 10), w("hey", 10, 400), w(" ", 400, 410), w("!", 410, 460)]);
    expectWellFormed(out);
    expect(texts(out).join("")).toBe("hey!");
  });
});
