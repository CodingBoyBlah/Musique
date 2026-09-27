import { useQuery } from "@tanstack/react-query";
import { getHomeFeed, type HomeItem } from "../../api/internal";
import { Shelf } from "./Shelf";
import { MediaTile } from "./MediaTile";

const ROUTE: Partial<Record<HomeItem["kind"], string>> = {
  playlist: "playlist",
  album: "album",
  artist: "artist",
  show: "show",
};

/* spotify's own home shelves - daily mixes, daylist, discover weekly, release
radar and the rest of made-for-you. best effort: if the feed can't be read,
this renders nothing and the local shelves below carry the page. */
export function HomeFeedShelves() {
  const { data } = useQuery({
    queryKey: ["library", "home-feed"],
    queryFn: () => getHomeFeed(Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"),
    staleTime: 30 * 60_000,
    retry: false,
  });
  if (!data) return null;
  return (
    <>
      {data.sections.map((section) => {
        // only things that have a page to open
        const items = section.items.filter((it) => ROUTE[it.kind]);
        if (items.length < 2) return null;
        return (
          <Shelf
            key={section.id}
            id={`home-feed-${section.id.replace(/[^a-zA-Z0-9]/g, "")}`}
            title={section.title}
            items={items}
            getKey={(it) => `${it.kind}-${it.id}`}
            renderItem={(it, i) => (
              <MediaTile
                to={`/${ROUTE[it.kind]}/${it.id}`}
                imageUrl={it.image_url}
                title={it.name}
                subtitle={it.subtitle}
                index={i}
                round={it.kind === "artist"}
              />
            )}
          />
        );
      })}
    </>
  );
}
