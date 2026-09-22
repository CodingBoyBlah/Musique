// YouTube InnerTube client identities used for anonymous stream extraction.
//
// Only a trimmed subset of Metrolist/innertubex's 30-profile catalogue is here,
// and the selection rule is narrow: we keep ONLY identities that return
// `adaptiveFormats[].url` in plaintext. Those are the ones with no signature
// timestamp, which means YouTube hands back a pre-signed CDN URL instead of a
// `signatureCipher` blob.
//
// That single property is what lets this whole feature exist without a
// JavaScript engine: the clients that require cipher deobfuscation (WEB_REMIX,
// TVHTML5, anything `useSignatureTimestamp = true`) also require a BotGuard PO
// token, and innertubex drags in QuickJS + an Android WebView to produce both.
// Anonymous WEB_REMIX player calls return UNPLAYABLE anyway, so there is
// nothing to gain from that path for a logged-out extraction.
//
// See findings.md "Free-Account Playback via YouTube Music" for the live probe
// results these profiles were chosen from.

/// An InnerTube client identity. Field names map 1:1 onto the `context.client`
/// object InnerTube expects.
#[derive(Debug, Clone, Copy)]
pub struct YtClient {
    pub name:         &'static str,
    pub version:      &'static str,
    /// Value for the `X-YouTube-Client-Name` header.
    pub id:           &'static str,
    pub user_agent:   &'static str,
    pub os_name:      Option<&'static str>,
    pub os_version:   Option<&'static str>,
    pub device_make:  Option<&'static str>,
    pub device_model: Option<&'static str>,
    pub android_sdk:  Option<&'static str>,
    pub platform:     Option<&'static str>,
    /// Whether `context.client.userAgent` is echoed inside the JSON body.
    /// Android clients are rejected without it.
    pub ua_in_body:   bool,
    /// Hit `music.youtube.com/youtubei/v1/player` rather than `www.`.
    pub music_origin: bool,
}

/// Extraction ladder, best first.
///
/// Ordering follows innertubex's `PlaybackClientCatalog` priorities restricted
/// to the profiles that are `AUTOMATIC`/anonymous-capable. `VISIONOS_0_1` sits
/// at the top there (priority 100) and matched that in our own probes: it
/// returned the widest format set (5 audio itags including Opus) and was the
/// only profile flagged as validated for anonymous playback end to end.
///
/// The rest are genuine fallbacks, not decoration - YouTube blocks client
/// identities periodically and without warning, so a single profile is a
/// single point of failure.
pub const EXTRACTION_LADDER: &[YtClient] = &[VISIONOS_0_1, VISIONOS_1_02, ANDROID_VR_1_61_48, ANDROID_VR_1_43_32];

pub const VISIONOS_0_1: YtClient = YtClient {
    name:         "VISIONOS",
    version:      "0.1",
    id:           "101",
    user_agent:   "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6) AppleWebKit/605.1.15 \
                   (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
    os_name:      Some("VISION_OS"),
    os_version:   Some("1.3"),
    device_make:  Some("Apple"),
    device_model: Some("RealityDevice14,1"),
    android_sdk:  None,
    platform:     Some("MOBILE"),
    ua_in_body:   false,
    music_origin: true,
};

pub const VISIONOS_1_02: YtClient = YtClient {
    name:         "VISIONOS",
    version:      "1.02",
    id:           "101",
    user_agent:   "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 \
                   (KHTML, like Gecko) Version/26.0 Safari/605.1.15",
    os_name:      Some("visionOS"),
    os_version:   Some("26.5.23O471"),
    device_make:  Some("Apple"),
    device_model: Some("RealityDevice17,1"),
    android_sdk:  None,
    platform:     None,
    ua_in_body:   false,
    music_origin: true,
};

pub const ANDROID_VR_1_61_48: YtClient = YtClient {
    name:         "ANDROID_VR",
    version:      "1.61.48",
    id:           "28",
    user_agent:   "com.google.android.apps.youtube.vr.oculus/1.61.48 \
                   (Linux; U; Android 12; en_US; Quest 3; Build/SQ3A.220605.009.A1; \
                   Cronet/132.0.6808.3)",
    os_name:      Some("Android"),
    os_version:   Some("12"),
    device_make:  Some("Oculus"),
    device_model: Some("Quest 3"),
    android_sdk:  Some("32"),
    platform:     None,
    ua_in_body:   true,
    music_origin: true,
};

// Older VR profile. Uses non-adaptive bitrate, which innertubex notes fixes
// audio stuttering on YT Music, so it is worth keeping as a distinct fallback
// rather than a near-duplicate of 1.61.48.
pub const ANDROID_VR_1_43_32: YtClient = YtClient {
    name:         "ANDROID_VR",
    version:      "1.43.32",
    id:           "28",
    user_agent:   "com.google.android.apps.youtube.vr.oculus/1.43.32 \
                   (Linux; U; Android 12; en_US; Quest 3; Build/SQ3A.220605.009.A1; \
                   Cronet/107.0.5284.2)",
    os_name:      Some("Android"),
    os_version:   Some("12"),
    device_make:  Some("Oculus"),
    device_model: Some("Quest 3"),
    android_sdk:  Some("32"),
    platform:     None,
    ua_in_body:   true,
    music_origin: true,
};

/// Search/browse identity. Distinct from the extraction ladder on purpose:
/// WEB_REMIX is useless for anonymous *playback* (returns UNPLAYABLE) but it is
/// the correct - and richest - client for anonymous YouTube Music *search*,
/// which is where the Spotify->YouTube matching data comes from.
pub const WEB_REMIX: YtClient = YtClient {
    name:         "WEB_REMIX",
    version:      "1.20260707.12.00",
    id:           "67",
    user_agent:   "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) \
                   Gecko/20100101 Firefox/140.0",
    os_name:      None,
    os_version:   None,
    device_make:  None,
    device_model: None,
    android_sdk:  None,
    platform:     None,
    ua_in_body:   false,
    music_origin: true,
};
