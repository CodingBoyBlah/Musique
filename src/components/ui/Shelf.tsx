import { SectionTitle } from "./SectionTitle";
import { useCarousel, CarouselControls, CarouselTrack } from "./Carousel";

// the tile every shelf uses (Album "More by", Artist, Search), so shelves line up app-wide
export const SHELF_TILE = "clamp(140px, 16vw, 175px)";

/* a titled horizontal shelf. Each owns its carousel so the arrows track their
own row. `extra` sits beside the arrows (eg a "Show all" that opens the full
grid). Renders nothing for an empty list. */
export function Shelf<T>({
  id,
  title,
  items,
  getKey,
  renderItem,
  extra,
}: {
  id: string;
  title: string;
  items: T[];
  getKey: (item: T) => string;
  renderItem: (item: T, index: number) => React.ReactNode;
  extra?: React.ReactNode;
}) {
  const carousel = useCarousel([items.length]);
  if (items.length === 0) return null;
  return (
    <section aria-labelledby={id}>
      <SectionTitle
        id={id}
        right={
          <>
            {extra}
            <CarouselControls carousel={carousel} label={title.toLowerCase()} />
          </>
        }
      >
        {title}
      </SectionTitle>
      <CarouselTrack
        carousel={carousel}
        label={title}
        items={items}
        getKey={getKey}
        itemWidth={SHELF_TILE}
        renderItem={renderItem}
      />
    </section>
  );
}
