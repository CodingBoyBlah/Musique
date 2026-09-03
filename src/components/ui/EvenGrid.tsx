import { useRef, useState, useLayoutEffect, useEffect, type ReactNode, type CSSProperties } from "react";
import { useReflowPulse } from "../../hooks/useReflowPulse";

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
  useReflowPulse();

  // Initialize with estimated width based on window to prevent layout flash
  const [containerWidth, setContainerWidth] = useState<number>(() => {
    if (typeof window !== "undefined") {
      return Math.max(320, window.innerWidth - 320);
    }
    return 800;
  });

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const measure = () => {
      const w = el.getBoundingClientRect().width;
      if (w > 0) setContainerWidth(w);
    };

    measure();

    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const w = entry.contentRect.width;
        if (w > 0) setContainerWidth(w);
      }
    });

    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Compute column count that comfortably fits the container width
  const fitCols = Math.floor((containerWidth + gap) / (minColWidth + gap));
  let cols = Math.min(maxCols, Math.max(minCols, fitCols));

  // If there are fewer items than calculated columns, adapt columns to items.length
  // so the available items fill the entire row without gaps
  if (items.length < cols && items.length >= minCols) {
    cols = items.length;
  }

  // Calculate complete rows
  const completeRows = Math.floor(items.length / cols);
  const targetRows = Math.min(maxRows, completeRows);

  let visibleCount: number;
  let effectiveCols = cols;

  if (targetRows > 0) {
    visibleCount = targetRows * cols;
  } else {
    // If we have fewer items than minCols (e.g. 1 item), show that item full row
    effectiveCols = Math.max(1, items.length);
    visibleCount = effectiveCols;
  }

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
        ...style,
      }}
    >
      {visibleItems.map((item, index) => {
        const key = getKey ? getKey(item, index) : index;
        return <div key={key} style={{ minWidth: 0 }}>{renderItem(item, index)}</div>;
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
  useReflowPulse();

  const [containerWidth, setContainerWidth] = useState<number>(() => {
    if (typeof window !== "undefined") {
      return Math.max(320, window.innerWidth - 320);
    }
    return 800;
  });

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const measure = () => {
      const w = el.getBoundingClientRect().width;
      if (w > 0) setContainerWidth(w);
    };
    measure();
    const ro = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0) setContainerWidth(entry.contentRect.width);
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const fitCols = Math.floor((containerWidth + gap) / (minColWidth + gap));
  const cols = Math.min(maxCols, Math.max(minCols, fitCols));
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
