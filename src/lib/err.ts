
export function errMsg(e: unknown): string {
  if (e == null) return "Something went wrong";
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  if (typeof e === "object") {
    const o = e as Record<string, unknown>;
    // AppError: { kind: "Network", message: "..." }
    if (typeof o.message === "string" && o.message) return o.message;
    if (typeof o.kind === "string") return o.kind;
    try {
      return JSON.stringify(e);
    } catch {
      /* give up */
    }
  }
  return String(e);
}
