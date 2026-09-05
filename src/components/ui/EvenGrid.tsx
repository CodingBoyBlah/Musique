import { useRef, useState, useLayoutEffect, useEffect, useMemo, type ReactNode, type CSSProperties } from "react";

interface EvenGridProps<T> {
  items: T[];
  minColWidth?: number;
  gap?: number;
  maxRows?: number;
  minCols?: number;
  maxCols?: number;
  renderItem: (item: T, index: number) => ReactNode;
  getKey?: (item: T, index: number) => string | number;
  style?: CSSProperties;
  className?: string;
}

interface GridMetrics {
  effectiveCols: number;
  visibleCount: number;
}

function calculateGridMetrics(
  width: number,
  itemCount: number,
  minColWidth: number,
  gap: number,
  maxRows: number,
  minCols: number,
  maxCols: number
): GridMetrics {
  const fitCols = Math.floor((width + gap) / (minColWidth + gap));
  let cols = Math.min(maxCols, Math.max(minCols, fitCols));

  if (itemCount < cols && itemCount >= minCols) {
    cols = itemCount;
  }

  const completeRows = Math.floor(itemCount / cols);
  const targetRows = Math.min(maxRows, completeRows);

  let visibleCount: number;
  let effectiveCols = cols;

  if (targetRows > 0) {
    visibleCount = targetRows * cols;
  } else {
    effectiveCols = Math.max(1, itemCount);
    visibleCount = effectiveCols;
  }

  return { effectiveCols, visibleCount };
}

export function EvenGrid<T>({
  items,
  minColWidth = 140,
  gap = 14,
  maxRows = 2,
  minCols = 2,
  maxCols = 8,
  renderItem,
  getKey,
  style,
  className,
}: EvenGridProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const lastWidthRef = useRef<number>(
    typeof window !== "undefined" ? Math.max(320, window.innerWidth - 320) : 800
  );

  const [metrics, setMetrics] = useState<GridMetrics>(() => {
    return calculateGridMetrics(
      lastWidthRef.current,
      items.length,
      minColWidth,
      gap,
      maxRows,
      minCols,
      maxCols
    );
  });

  const paramsRef = useRef({
    itemsLength: items.length,
    minColWidth,
    gap,
    maxRows,
    minCols,
    maxCols,
  });
  paramsRef.current = {
    itemsLength: items.length,
    minColWidth,
    gap,
    maxRows,
    minCols,
    maxCols,
  };

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    let rafId = 0;
    const measure = (w: number) => {
      if (w <= 0) return;
      lastWidthRef.current = w;
      const { itemsLength, minColWidth, gap, maxRows, minCols, maxCols } = paramsRef.current;
      const next = calculateGridMetrics(w, itemsLength, minColWidth, gap, maxRows, minCols, maxCols);
      setMetrics((prev) => {
        if (prev.effectiveCols === next.effectiveCols && prev.visibleCount === next.visibleCount) {
          return prev;
        }
        return next;
      });
    };

    measure(el.getBoundingClientRect().width);

    const ro = new ResizeObserver((entries) => {
      if (rafId) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        for (const entry of entries) {
          measure(entry.contentRect.width);
        }
      });
    });

    ro.observe(el);
    return () => {
      if (rafId) cancelAnimationFrame(rafId);
      ro.disconnect();
    };
  }, []);

  // Update metrics synchronously during render when input parameters change
  const derivedMetrics = useMemo(() => {
    return calculateGridMetrics(
      lastWidthRef.current,
      items.length,
      minColWidth,
      gap,
      maxRows,
      minCols,
      maxCols
    );
  }, [items.length, minColWidth, gap, maxRows, minCols, maxCols]);

  useEffect(() => {
    setMetrics((prev) => {
      if (prev.effectiveCols === derivedMetrics.effectiveCols && prev.visibleCount === derivedMetrics.visibleCount) {
        return prev;
      }
      return derivedMetrics;
    });
  }, [derivedMetrics]);

  const { effectiveCols, visibleCount } = metrics;
  const visibleItems = items.slice(0, visibleCount);

  return (
    <div
      ref={containerRef}
      className={className}
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${effectiveCols}, minmax(0, 1fr))`,
        gap,
        width: "100%",
        contain: "layout style",
        ...style,
      }}
    >
      {visibleItems.map((item, index) => {
        const key = getKey ? getKey(item, index) : index;
        return (
          <div
            key={key}
            style={{
              minWidth: 0,
              contain: "layout style",
            }}
          >
            {renderItem(item, index)}
          </div>
        );
      })}
    </div>
  );
}

export function EvenGridSkeleton({
  minColWidth = 140,
  gap = 14,
  maxRows = 1,
  minCols = 2,
  maxCols = 8,
  aspectRatio = "1 / 1",
  borderRadius = 12,
}: {
  minColWidth?: number;
  gap?: number;
  maxRows?: number;
  minCols?: number;
  maxCols?: number;
  aspectRatio?: string;
  borderRadius?: number;
}) {
  const containerRef = useRef<HTMLDivElement>(null);

  const [cols, setCols] = useState<number>(() => {
    const initialWidth = typeof window !== "undefined" ? Math.max(320, window.innerWidth - 320) : 800;
    const fitCols = Math.floor((initialWidth + gap) / (minColWidth + gap));
    return Math.min(maxCols, Math.max(minCols, fitCols));
  });

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const measure = (w: number) => {
      if (w <= 0) return;
      const fitCols = Math.floor((w + gap) / (minColWidth + gap));
      const nextCols = Math.min(maxCols, Math.max(minCols, fitCols));
      setCols((prev) => (prev === nextCols ? prev : nextCols));
    };

    measure(el.getBoundingClientRect().width);

    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        measure(entry.contentRect.width);
      }
    });

    ro.observe(el);
    return () => ro.disconnect();
  }, [gap, minColWidth, maxCols, minCols]);

  const totalCount = cols * maxRows;

  return (
    <div
      ref={containerRef}
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
        gap,
        width: "100%",
      }}
    >
      {Array.from({ length: totalCount }).map((_, i) => (
        <div
          key={i}
          style={{
            aspectRatio,
            borderRadius,
            background: "var(--color-surface)",
          }}
        />
      ))}
    </div>
  );
}
