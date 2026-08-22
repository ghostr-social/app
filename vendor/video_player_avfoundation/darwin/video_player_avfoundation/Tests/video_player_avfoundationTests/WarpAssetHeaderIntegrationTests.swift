import AVFoundation
import Testing
import video_player_avfoundation_objc

@testable import video_player_avfoundation

private let attemptHeader = "X-Ghostr-Playback-Attempt"
private let attemptToken = "AAAAAAAAAAAAAAAAAAAAAA"

@MainActor @Suite struct WarpAssetHeaderIntegrationTests {
  @Test func reservedHeaderNeverReachesAVURLAsset() throws {
    let factory = WarpTestAVFactory()
    let plugin = makeWarpTestPlugin(factory: factory)

    _ = try plugin.createTexturePlayer(
      options: CreationOptions(
        uri: "https://flutter.dev/video.mp4",
        httpHeaders: [attemptHeader: attemptToken, "Authorization": "Bearer normal"]
      ))

    let options = try #require(factory.assetOptions)
    let headers = try #require(
      options["AVURLAssetHTTPHeaderFieldsKey"] as? [String: String])
    #expect(headers == ["Authorization": "Bearer normal"])
  }

  @Test func noTokenPreservesOrdinaryPluginBehavior() throws {
    let factory = WarpTestAVFactory()
    let registry = WarpTestTextureRegistry()
    var callbackWasInstalled = true
    registry.onRegister = { texture in
      let player = texture as! FVPTextureBasedVideoPlayer
      callbackWasInstalled = player.warpFirstFrameRenderedCallback != nil
    }

    _ = try makeWarpTestPlugin(factory: factory, registry: registry)
      .createTexturePlayer(
        options: CreationOptions(
          uri: "https://flutter.dev/video.mp4",
          httpHeaders: ["Range": "bytes=0-99"]
        ))

    #expect(!callbackWasInstalled)
  }
}
