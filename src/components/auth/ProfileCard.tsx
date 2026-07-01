interface Props {
  displayName: string | null;
  email:       string | null;
  product:     string | null;
  imageUrl:    string | null;
}

export function ProfileCard({ displayName, email, product, imageUrl }: Props) {
  const productLabel = product
    ? product.charAt(0).toUpperCase() + product.slice(1)
    : null;

  return (
    <div
      className="flex items-center gap-4 p-4 rounded-xl mb-6"
      style={{ background: "#282828" }}
    >
      {imageUrl ? (
        <img
          src={imageUrl}
          alt="Profile"
          referrerPolicy="no-referrer"
          className="w-14 h-14 rounded-full object-cover flex-shrink-0"
        />
      ) : (
        <div
          className="w-14 h-14 rounded-full flex items-center justify-center flex-shrink-0 text-2xl"
          style={{ background: "#3e3e3e" }}
        >
          👤
        </div>
      )}

      <div className="flex flex-col gap-1 min-w-0">
        <span className="font-semibold text-base truncate">
          {displayName ?? "Spotify User"}
        </span>
        {email && (
          <span className="text-sm truncate" style={{ color: "#b3b3b3" }}>
            {email}
          </span>
        )}
        {productLabel && (
          <span
            className="text-xs px-2 py-0.5 rounded-full font-semibold self-start"
            style={{
              background: product === "premium" ? "#1db954" : "#535353",
              color:      product === "premium" ? "#000"     : "#fff",
            }}
          >
            {productLabel}
          </span>
        )}
      </div>
    </div>
  );
}
