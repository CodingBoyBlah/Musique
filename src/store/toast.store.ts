import { create } from "zustand";

export interface Toast {
  id:   number;
  text: string;
}

interface ToastStore {
  toasts: Toast[];
  push:   (text: string) => void;
  remove: (id: number) => void;
}

let seq = 0;

// tiny transient toast queue. not persisted, purely ephemeral UI feedback.
export const useToastStore = create<ToastStore>((set) => ({
  toasts: [],
  push: (text) => {
    const id = ++seq;
    set((s) => ({ toasts: [...s.toasts, { id, text }] }));
    // auto-dismiss after 2.6s
    setTimeout(() => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })), 2600);
  },
  remove: (id) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
}));

// convenience for non-component call sites
export const toast = (text: string) => useToastStore.getState().push(text);
