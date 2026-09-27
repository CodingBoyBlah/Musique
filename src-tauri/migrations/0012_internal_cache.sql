-- Cache for everything pulled off Spotify's internal endpoints (spclient
-- extended-metadata, credits, canvas, radio, pathfinder...).
--
-- One generic table rather than one per feature: every internal payload is
-- "some bytes for (entity, kind)", and they all want the same thing - serve it
-- again without a round trip while it's fresh, and serve it stale when the
-- endpoint is down or we're offline. `kind` is the ExtensionKind name for
-- extended-metadata payloads, or a short feature tag ("credits", "canvas", ...).
CREATE TABLE IF NOT EXISTS extension_cache (
    entity_uri TEXT    NOT NULL,
    kind       TEXT    NOT NULL,
    payload    BLOB    NOT NULL,
    fetched_at INTEGER NOT NULL,
    PRIMARY KEY (entity_uri, kind)
);

CREATE INDEX IF NOT EXISTS idx_extension_cache_fetched ON extension_cache (fetched_at);
