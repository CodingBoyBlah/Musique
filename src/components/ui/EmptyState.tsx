import { motion } from "framer-motion";

interface EmptyStateProps {
  icon?: React.ReactNode;
  iconContainerStyle?: React.CSSProperties;
  title: string;
  description?: string;
  hint?: string;
  action?: React.ReactNode;
  style?: React.CSSProperties;
}

export function EmptyState({
  icon,
  iconContainerStyle,
  title,
  description,
  hint,
  action,
  style,
}: EmptyStateProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25, ease: [0.23, 1, 0.32, 1] }}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        textAlign: "center",
        padding: "clamp(48px, 10vh, 80px) 20px",
        gap: 12,
        maxWidth: 420,
        margin: "0 auto",
        ...style,
      }}
    >
      {icon && (
        <div
          style={{
            color: "var(--color-text-dim)",
            opacity: 0.5,
            marginBottom: 4,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            ...iconContainerStyle,
          }}
        >
          {icon}
        </div>
      )}
      <h2
        style={{
          margin: 0,
          fontSize: 17,
          fontWeight: 700,
          letterSpacing: "-0.01em",
          color: "var(--color-text-hi)",
        }}
      >
        {title}
      </h2>
      {description && (
        <p
          style={{
            margin: 0,
            fontSize: 13.5,
            lineHeight: 1.5,
            color: "var(--color-text-dim)",
            maxWidth: 340,
          }}
        >
          {description}
        </p>
      )}
      {action && <div style={{ marginTop: 4 }}>{action}</div>}
      {hint && (
        <p
          style={{
            margin: 0,
            fontSize: 12,
            color: "var(--color-text-dim)",
            opacity: 0.7,
            maxWidth: 320,
          }}
        >
          {hint}
        </p>
      )}
    </motion.div>
  );
}
