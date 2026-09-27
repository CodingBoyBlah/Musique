import { create } from "zustand";

// the four kinds of feedback, minus warning (nothing warns yet). an error must
// never wear the success tick.
export type ToastKind = "success" | "error" | "info";

export interface ToastAction {
  label:   string;
  onClick: () => void;
}

export interface Toast {
  id:      number;
  text:    string;
  kind:    ToastKind;
  action?: ToastAction;
}

export interface ToastOptions {
  kind?:   ToastKind;
  action?: ToastAction;
}

interface ToastStore {
  toasts: Toast[];
  push:   (text: string, opts?: ToastOptions) => number;
  remove: (id: number) => void;
}

let seq = 0;

// tiny transient toast queue. not persisted, purely ephemeral UI feedback.
export const useToastStore = create<ToastStore>((set) => ({
  toasts: [],
  push: (text, opts) => {
    const id = ++seq;
    const t: Toast = { id, text, kind: opts?.kind ?? "success", action: opts?.action };
    set((s) => ({ toasts: [...s.toasts, t] }));
    // an undo needs longer than a glance to reach
    const ttl = t.action ? 5000 : 2600;
    setTimeout(() => set((s) => ({ toasts: s.toasts.filter((x) => x.id !== id) })), ttl);
    return id;
  },
  remove: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

// convenience for non-component call sites
export const toast = Object.assign(
  (text: string, opts?: ToastOptions) => useToastStore.getState().push(text, opts),
  {
    error: (text: string) => useToastStore.getState().push(text, { kind: "error" }),
    info:  (text: string) => useToastStore.getState().push(text, { kind: "info" }),
  },
);
