import { Link } from "react-router-dom";
import { motion } from "framer-motion";
import { useQuery } from "@tanstack/react-query";
import {
  ExternalLink,
  Users,
  Globe,
  Mail,
  ShieldCheck,
  ShieldAlert,
  BadgeCheck,
  LogOut,
  Settings as SettingsIcon,
} from "lucide-react";
import { getProfile } from "../api/auth";
import { useAuth } from "../hooks/useAuth";
import { useAuthStore } from "../store/auth.store";
import { useTopArtists } from "../hooks/useLibrary";
import { ArtistCard, ArtistGrid } from "../components/ui/ArtistCard";
import { Loader } from "../components/ui/Loader";
import type { Profile as ProfileT } from "../types/spotify";

function Stat({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: React.ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 12,
        padding: "14px 16px",
        borderRadius: 12,
        background: "var(--color-surface)",
        border: "1px solid var(--color-border)",
      }}
    >
      <span
        style={{ display: "flex", color: "var(--color-accent)", flexShrink: 0 }}
      >
        {icon}
      </span>
      <div style={{ minWidth: 0 }}>
        <div
          style={{
            fontSize: 11,
            fontWeight: 700,
            letterSpacing: "0.04em",
            textTransform: "uppercase",
            color: "var(--color-text-dim)",
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "var(--color-text-hi)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {value}
        </div>
      </div>
    </div>
  );
}

export default function Profile() {
  const loggedIn = useAuthStore((s) => s.loggedIn);
  const { logout, loggingOut } = useAuth();
  const { data, isLoading } = useQuery<ProfileT>({
    queryKey: ["profile"],
    queryFn: getProfile,
    enabled: loggedIn,
    staleTime: 5 * 60_000,
  });
  const { data: topArtists = [] } = useTopArtists("medium_term");

  if (!loggedIn) {
    return (
      <div>
        <h1
          style={{
            margin: "0 0 12px",
            fontSize: 26,
            fontWeight: 700,
            color: "var(--color-text-hi)",
          }}
        >
          Profile
        </h1>
        <p style={{ color: "var(--color-text-dim)" }}>
          <Link
            to="/"
            style={{ color: "var(--color-accent)", textDecoration: "none" }}
          >
            Login
          </Link>{" "}
          to view your profile.
        </p>
      </div>
    );
  }

  if (isLoading || !data) return <Loader label="Loading profile" />;

  const premium = (data.product ?? "").toLowerCase() === "premium";

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 28,
        maxWidth: 760,
      }}
    >
      {/* Hero */}
      <motion.section
        initial={{ opacity: 0, y: 14 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: [0.23, 1, 0.32, 1] }}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "clamp(16px, 2.4vw, 24px)",
          flexWrap: "wrap",
        }}
      >
        {data.image_url ? (
          <img
            src={data.image_url}
            alt={data.display_name ?? ""}
            style={{
              width: 132,
              height: 132,
              borderRadius: "50%",
              objectFit: "cover",
              flexShrink: 0,
              boxShadow: "0 10px 30px rgba(0,0,0,0.45)",
            }}
          />
        ) : (
          <div
            style={{
              width: 132,
              height: 132,
              borderRadius: "50%",
              background: "var(--color-surface-2)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              flexShrink: 0,
            }}
          >
            <Users size={48} style={{ color: "var(--color-text-dim)" }} />
          </div>
        )}
        <div
          style={{
            minWidth: 0,
            display: "flex",
            flexDirection: "column",
            gap: 10,
          }}
        >
          <span
            style={{
              fontSize: 11,
              fontWeight: 700,
              letterSpacing: "0.06em",
              textTransform: "uppercase",
              color: "var(--color-text-dim)",
            }}
          >
            Profile
          </span>
          <h1
            style={{
              margin: 0,
              fontSize: "clamp(28px, 5vw, 42px)",
              fontWeight: 800,
              letterSpacing: "-0.02em",
              color: "var(--color-text-hi)",
              lineHeight: 1.05,
            }}
          >
            {data.display_name ?? "Spotify User"}
          </h1>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              flexWrap: "wrap",
            }}
          >
            {data.product && (
              <span
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 5,
                  fontSize: 11,
                  fontWeight: 700,
                  letterSpacing: "0.04em",
                  textTransform: "uppercase",
                  color: premium
                    ? "var(--color-accent-text)"
                    : "var(--color-text-hi)",
                  background: premium
                    ? "var(--color-accent)"
                    : "var(--color-surface-2)",
                  padding: "3px 10px",
                  borderRadius: 99,
                }}
              >
                {premium && <BadgeCheck size={12} strokeWidth={2.5} />}
                {data.product}
              </span>
            )}
            <span style={{ fontSize: 13, color: "var(--color-text-dim)" }}>
              {data.followers.toLocaleString()} follower
              {data.followers === 1 ? "" : "s"}
            </span>
            {data.spotify_url && (
              <a
                href={data.spotify_url}
                target="_blank"
                rel="noreferrer"
                style={{
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 4,
                  fontSize: 13,
                  fontWeight: 500,
                  color: "var(--color-accent)",
                  textDecoration: "none",
                }}
              >
                Open in Spotify <ExternalLink size={12} strokeWidth={2.5} />
              </a>
            )}
          </div>
        </div>
      </motion.section>

      {/* Stats grid */}
      <section
        style={{
          display: "grid",
          gap: "clamp(10px, 1.2vw, 12px)",
          gridTemplateColumns:
            "repeat(auto-fill, minmax(clamp(170px, 22vw, 240px), 1fr))",
        }}
      >
        <Stat
          icon={<Users size={18} />}
          label="Followers"
          value={data.followers.toLocaleString()}
        />
        {data.country && (
          <Stat
            icon={<Globe size={18} />}
            label="Country"
            value={data.country}
          />
        )}
        {data.email && (
          <Stat icon={<Mail size={18} />} label="Email" value={data.email} />
        )}
        <Stat
          icon={
            data.explicit_filter_enabled ? (
              <ShieldCheck size={18} />
            ) : (
              <ShieldAlert size={18} />
            )
          }
          label="Explicit content"
          value={data.explicit_filter_enabled ? "Filtered" : "Allowed"}
        />
      </section>

      {/* Account actions */}
      <section style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
        <Link
          to="/settings"
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 8,
            height: 40,
            padding: "0 18px",
            borderRadius: 99,
            textDecoration: "none",
            border: "1px solid var(--color-border)",
            background: "var(--color-surface)",
            color: "var(--color-text-hi)",
            fontSize: 13.5,
            fontWeight: 600,
          }}
        >
          <SettingsIcon size={15} strokeWidth={2} /> Spotify API settings
        </Link>
        <button
          onClick={() => logout()}
          disabled={loggingOut}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 8,
            height: 40,
            padding: "0 18px",
            borderRadius: 99,
            border: "1px solid var(--color-border)",
            background: "var(--color-surface)",
            color: "var(--color-danger)",
            fontSize: 13.5,
            fontWeight: 600,
            cursor: loggingOut ? "default" : "pointer",
            opacity: loggingOut ? 0.5 : 1,
          }}
          onMouseEnter={(e) => {
            if (!loggingOut)
              (e.currentTarget as HTMLButtonElement).style.background =
                "var(--color-surface-2)";
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.background =
              "var(--color-surface)";
          }}
        >
          <LogOut size={15} strokeWidth={2} />{" "}
          {loggingOut ? "Logging out…" : "Log out"}
        </button>
      </section>

      {/* Top artists glance */}
      {topArtists.length > 0 && (
        <section>
          <div
            style={{
              display: "flex",
              alignItems: "baseline",
              justifyContent: "space-between",
              margin: "0 0 14px",
            }}
          >
            <h2
              style={{
                margin: 0,
                fontSize: 18,
                fontWeight: 700,
                color: "var(--color-text-hi)",
              }}
            >
              Your top artists
            </h2>
            <Link
              to="/"
              style={{
                fontSize: 12,
                color: "var(--color-text-dim)",
                textDecoration: "none",
              }}
            >
              See all on Home
            </Link>
          </div>
          <ArtistGrid>
            {topArtists.slice(0, 6).map((a) => (
              <ArtistCard key={a.id} artist={a} />
            ))}
          </ArtistGrid>
        </section>
      )}
    </div>
  );
}
