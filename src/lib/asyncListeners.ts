/** Release each successful registration, including ones arriving after cleanup. */
export function cleanupAsyncListeners(registrations: readonly Promise<() => void>[]): () => void {
  const unlisteners: Array<() => void> = [];
  let disposed = false;
  for (const registration of registrations) {
    registration.then((unlisten) => {
      if (disposed) unlisten();
      else unlisteners.push(unlisten);
    }).catch(() => {});
  }
  return () => {
    if (disposed) return;
    disposed = true;
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners.length = 0;
  };
}
