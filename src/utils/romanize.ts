
export type Script = "japanese" | "han" | "hangul" | "cyrillic" | "latin" | "other";

let pinyinFn: ((t: string, o: object) => string) | null = null;
async function loadPinyin(): Promise<(t: string, o: object) => string> {
  if (!pinyinFn) pinyinFn = (await import("pinyin-pro")).pinyin as (t: string, o: object) => string;
  return pinyinFn;
}

let toRomajiFn: ((s: string) => string) | null = null;
async function loadWanakana(): Promise<(s: string) => string> {
  if (!toRomajiFn) toRomajiFn = (await import("wanakana")).toRomaji;
  return toRomajiFn;
}

// script detection

const RANGES: { script: Script; re: RegExp }[] = [
  { script: "japanese", re: /[぀-ヿ]/ },               // kana
  { script: "han",      re: /[㐀-䶿一-鿿]/ },   // CJK ideographs
  { script: "hangul",   re: /[가-힣ᄀ-ᇿ㄰-㆏]/ },
  { script: "cyrillic", re: /[Ѐ-ӿ]/ },
];

function countMatches(text: string, re: RegExp): number {
  const g = new RegExp(re.source, "gu");
  return (text.match(g) ?? []).length;
}

export function detectLyricScript(texts: string[]): Script {
  const blob = texts.join("\n");
  const counts: Record<string, number> = {};
  for (const { script, re } of RANGES) counts[script] = countMatches(blob, re);

  if ((counts.japanese ?? 0) > 0) return "japanese";

  const ranked = (["hangul", "han", "cyrillic"] as Script[])
    .map((s) => ({ s, n: counts[s] ?? 0 }))
    .sort((a, b) => b.n - a.n);
  if (ranked[0] && ranked[0].n > 0) return ranked[0].s;

  return /[A-Za-z]/.test(blob) ? "latin" : "other";
}

export function canRomanize(script: Script): boolean {
  return script === "japanese" || script === "han" || script === "hangul" || script === "cyrillic";
}

export function scriptLabel(script: Script): string {
  switch (script) {
    case "japanese": return "Romaji";
    case "han":      return "Pinyin";
    case "hangul":   return "Romaja";
    case "cyrillic": return "Latin";
    default:         return "Pronunciation";
  }
}

// korean: revised romanization (per-syllable + silent-onset liaison)

const KO_CHO = ["g","kk","n","d","tt","r","m","b","pp","s","ss","","j","jj","ch","k","t","p","h"];
const KO_JUNG = ["a","ae","ya","yae","eo","e","yeo","ye","o","wa","wae","oe","yo","u","wo","we","wi","yu","eu","ui","i"];
// final (batchim) as pronounced when NOT carried to a following silent onset
const KO_JONG = ["","k","k","k","n","n","n","t","l","k","m","l","l","l","p","l","m","p","p","t","t","ng","t","t","k","t","p","t"];
// when carried into a following silent (ㅇ) onset the batchim links like this
const KO_JONG_LINK = ["","g","kk","ks","n","nj","nh","d","l","lg","lm","lb","ls","lt","lp","lh","m","b","bs","s","ss","ng","j","ch","k","t","p","h"];

function decomposeHangul(code: number): [number, number, number] | null {
  if (code < 0xac00 || code > 0xd7a3) return null;
  const i = code - 0xac00;
  return [Math.floor(i / 588), Math.floor(i / 28) % 21, i % 28];
}

function romanizeKorean(text: string): string {
  const chars = Array.from(text);
  let out = "";
  let carriedOnset: string | null = null; // batchim carried into a silent onset

  for (let i = 0; i < chars.length; i++) {
    const cur = decomposeHangul(chars[i].codePointAt(0)!);
    if (!cur) { out += chars[i]; carriedOnset = null; continue; }
    const [cho, jung, jong] = cur;

    const next = i + 1 < chars.length ? decomposeHangul(chars[i + 1].codePointAt(0)!) : null;
    const nextIsSilentOnset = next != null && next[0] === 11; // ㅇ

    const onset = carriedOnset != null ? carriedOnset : KO_CHO[cho];
    const nucleus = KO_JUNG[jung];

    let coda: string;
    if (jong !== 0 && nextIsSilentOnset) {
      coda = "";
      carriedOnset = KO_JONG_LINK[jong];
    } else {
      coda = KO_JONG[jong];
      carriedOnset = null;
    }

    out += onset + nucleus + coda;
  }
  return out;
}

// cyrillic

const CYR: Record<string, string> = {
  "а":"a","б":"b","в":"v","г":"g","д":"d","е":"e","ё":"yo",
  "ж":"zh","з":"z","и":"i","й":"y","к":"k","л":"l","м":"m",
  "н":"n","о":"o","п":"p","р":"r","с":"s","т":"t","у":"u",
  "ф":"f","х":"kh","ц":"ts","ч":"ch","ш":"sh","щ":"shch",
  "ъ":"","ы":"y","ь":"","э":"e","ю":"yu","я":"ya",
};
function romanizeCyrillic(text: string): string {
  let out = "";
  for (const ch of text) {
    const lower = ch.toLowerCase();
    const mapped = CYR[lower];
    if (mapped === undefined) { out += ch; continue; }
    out += ch === lower || mapped === "" ? mapped : mapped.charAt(0).toUpperCase() + mapped.slice(1);
  }
  return out;
}

// japanese: kuromoji + wanakana

type Token = { surface_form: string; reading?: string };
type Tokenizer = { tokenize: (s: string) => Token[] };

let jaTokenizer: Tokenizer | null = null;
let jaPromise: Promise<Tokenizer> | null = null;

function getJaTokenizer(): Promise<Tokenizer> {
  if (jaTokenizer) return Promise.resolve(jaTokenizer);
  if (!jaPromise) {
    jaPromise = import("kuromoji").then(
      (mod) =>
        new Promise<Tokenizer>((resolve, reject) => {
          const kuromoji = ((mod as { default?: unknown }).default ?? mod) as {
            builder: (o: { dicPath: string }) => { build: (cb: (e: Error | null, t: Tokenizer) => void) => void };
          };
          // dict served from /dict (public/dict/*.dat.gz). kuromoji's `browser`
          // field swaps in the XHR loader automatically.
          kuromoji.builder({ dicPath: "/dict" }).build((err, tok) => (err ? reject(err) : resolve(tok)));
        }),
    );
  }
  return jaPromise;
}

const JA_PUNCT = /^[\s　-〿＀-￯.,!?…~]+$/u;

function romanizeJapanese(text: string, tok: Tokenizer, toRomaji: (s: string) => string): string {
  
  const parts: string[] = [];
  for (const t of tok.tokenize(text)) {
    if (JA_PUNCT.test(t.surface_form)) continue;
    const reading = t.reading && t.reading !== "*" ? t.reading : t.surface_form;
    const r = toRomaji(reading).trim();
    if (r) parts.push(r);
  }
  return parts.join(" ").replace(/\s+/g, " ").trim();
}

// public API

// romanize every line. returns one string per input line (same length).
export async function romanizeLines(texts: string[], script: Script): Promise<string[]> {
  switch (script) {
    case "han": {
      const pinyin = await loadPinyin();
      return texts.map((t) => pinyin(t, { toneType: "symbol", type: "string", nonZh: "consecutive" }).trim());
    }
    case "hangul":
      return texts.map(romanizeKorean);
    case "cyrillic":
      return texts.map(romanizeCyrillic);
    case "japanese": {
      const [tok, toRomaji] = await Promise.all([getJaTokenizer(), loadWanakana()]);
      return texts.map((t) => (t ? romanizeJapanese(t, tok, toRomaji) : ""));
    }
    default:
      return texts.slice();
  }
}
