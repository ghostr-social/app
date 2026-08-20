import CoreVideo
import Testing
import video_player_avfoundation_objc

@testable import video_player_avfoundation

@MainActor @Suite struct WarpTextureFirstFrameTests {
  @Test func reportsFirstNewNonNullPixelBufferExactlyOnce() {
    let pixels = WarpTestPixelBufferSource()
    let player = makeWarpTexturePlayer(pixels: pixels)
    var reports = 0
    player.warpFirstFrameRenderedCallback = { reports += 1 }

    player.copyPixelBuffer()
    #expect(reports == 0)
    pixels.pixelBuffer = makeWarpPixelBuffer()
    player.copyPixelBuffer()
    pixels.pixelBuffer = makeWarpPixelBuffer()
    player.copyPixelBuffer()

    #expect(reports == 1)
  }

  @Test func neverReportsAfterDispose() {
    let pixels = WarpTestPixelBufferSource()
    let player = makeWarpTexturePlayer(pixels: pixels)
    var reports = 0
    player.warpFirstFrameRenderedCallback = { reports += 1 }
    var error: FlutterError?
    player.disposeWithError(&error)

    pixels.pixelBuffer = makeWarpPixelBuffer()
    player.copyPixelBuffer()

    #expect(error == nil)
    #expect(reports == 0)
  }

  @Test func pluginInstallsCallbackBeforeTextureRegistrationCompletes() throws {
    let registry = WarpTestTextureRegistry()
    var callbackWasInstalled = false
    registry.onRegister = { texture in
      let player = texture as! FVPTextureBasedVideoPlayer
      callbackWasInstalled = player.warpFirstFrameRenderedCallback != nil
    }

    _ = try makeWarpTestPlugin(factory: WarpTestAVFactory(), registry: registry)
      .createTexturePlayer(
        options: CreationOptions(
          uri: "https://flutter.dev/video.mp4",
          httpHeaders: ["X-Ghostr-Playback-Attempt": "AAAAAAAAAAAAAAAAAAAAAA"]
        ))

    #expect(callbackWasInstalled)
  }
}
