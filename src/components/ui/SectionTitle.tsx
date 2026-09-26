/* the one section heading style. every page's h2 ("Popular", "Albums",
"Jump back in", "More by …") used to pick its own size and tracking; this is
the .t-section role, sentence case, with an optional control cluster on the
right (range picker, carousel arrows, "Show all"). */
export function SectionTitle({
  children,
  right,
  as: Tag = "h2",
  id,
  style,
}: {
  children: React.ReactNode;
  right?: React.ReactNode;
  as?: "h2" | "h3";
  id?: string;
  style?: React.CSSProperties;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: 12,
        flexWrap: "wrap",
        rowGap: 8,
        margin: "0 0 16px",
        ...style,
      }}
    >
      <Tag
        id={id}
        className="t-section"
        style={{
          margin: 0,
          // steps down to 18px in narrow windows, in rem so it follows the root size
          fontSize: "clamp(1.286rem, 2.2vw, var(--type-section-size))",
          color: "var(--color-text-hi)",
          textWrap: "balance",
          minWidth: 0,
        } as React.CSSProperties}
      >
        {children}
      </Tag>
      {right && <div style={{ display: "flex", alignItems: "center", gap: 8 }}>{right}</div>}
    </div>
  );
}

// the "Show all" affordance next to a section title
export function ShowAllButton({ onClick, label = "Show all" }: { onClick: () => void; label?: string }) {
  return (
    <button
      type="button"
      className="btn-text t-caption"
      onClick={onClick}
      style={{ fontSize: 12.5, fontWeight: 600, padding: "4px 8px", borderRadius: 6 }}
    >
      {label}
    </button>
  );
}
