/// A regular long-form video, generally presented in a horizontal format.
const REGULAR_NORMAL_VIDEO_KIND: u16 = 21;

/// A regular short-form vertical video, such as a Reel, Short, or Story.
const REGULAR_SHORT_VIDEO_KIND: u16 = 22;

/// An addressable normal video whose metadata can be updated without republishing it.
const ADDRESSABLE_NORMAL_VIDEO_KIND: u16 = 34_235;

/// An addressable short-form vertical video whose metadata can be updated in place.
const ADDRESSABLE_SHORT_VIDEO_KIND: u16 = 34_236;

// /// An addressable event used to announce and manage a live stream.
// ///
// /// It carries information such as participants, host, and streaming status.
// pub const LIVE_STREAMING_EVENT_KIND: u16 = 30_311;

pub const VIDEO_KINDS: [u16; 4] = [
    REGULAR_NORMAL_VIDEO_KIND,
    REGULAR_SHORT_VIDEO_KIND,
    ADDRESSABLE_NORMAL_VIDEO_KIND,
    ADDRESSABLE_SHORT_VIDEO_KIND,
];
