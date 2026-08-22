// swift-tools-version: 5.9

import PackageDescription

let package = Package(
  name: "WarpVideoPlayerNativeTests",
  platforms: [.macOS(.v10_15)],
  targets: [
    .target(
      name: "WarpVideoPlayerCore",
      path: "video_player_avfoundation/Sources/video_player_avfoundation",
      exclude: [
        "NativeVideoViewFactory.swift",
        "Resources",
        "WarpFirstFrameEventStream.swift",
        "WarpPreparedPlayerItem.swift",
        "VideoPlayerPlugin.swift",
        "VideoPlayerPluginMessages.g.swift",
      ],
      sources: [
        "WarpFirstFrameReplayBuffer.swift",
        "WarpPlaybackAttemptHeaders.swift",
      ]
    ),
    .testTarget(
      name: "WarpVideoPlayerCoreTests",
      dependencies: ["WarpVideoPlayerCore"],
      path: "WarpTests"
    ),
  ]
)
