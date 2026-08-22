import AVFoundation
import video_player_avfoundation_objc

final class WarpTestPixelBufferSource: NSObject, FVPPixelBufferSource {
  let videoOutput = AVPlayerItemVideoOutput(pixelBufferAttributes: nil)
  var pixelBuffer: CVPixelBuffer?
  func itemTime(forHostTime hostTimeInSeconds: CFTimeInterval) -> CMTime { .zero }
  func hasNewPixelBuffer(forItemTime itemTime: CMTime) -> Bool { pixelBuffer != nil }
  func copyPixelBuffer(
    forItemTime itemTime: CMTime,
    itemTimeForDisplay: UnsafeMutablePointer<CMTime>?
  ) -> CVPixelBuffer? {
    defer { pixelBuffer = nil }
    return pixelBuffer
  }
}

final class WarpTestAVFactory: NSObject, FVPAVFactory {
  private let base: FVPDefaultAVFactory
  let item: FVPAVPlayerItem
  let pixels: WarpTestPixelBufferSource
  private(set) var assetOptions: [String: Any]?

  init(pixels: WarpTestPixelBufferSource = WarpTestPixelBufferSource()) {
    let base = FVPDefaultAVFactory()
    let url = URL(string: "https://flutter.dev/video.mp4")!
    let asset = base.urlAsset(with: url, options: nil)
    self.base = base
    self.item = base.playerItem(with: asset)
    self.pixels = pixels
  }
  func urlAsset(with url: URL, options: [String: Any]?) -> FVPAVAsset {
    assetOptions = options
    return item.asset
  }
  func playerItem(with asset: FVPAVAsset) -> FVPAVPlayerItem { item }
  func player(with playerItem: FVPAVPlayerItem) -> AVPlayer {
    base.player(with: playerItem)
  }
  func videoOutput(outputSettings: [String: Any]) -> FVPPixelBufferSource { pixels }
  #if os(iOS)
    func sharedAudioSession() -> FVPAVAudioSession { WarpTestAudioSession() }
  #endif
}

func makeWarpTexturePlayer(
  pixels: WarpTestPixelBufferSource
) -> FVPTextureBasedVideoPlayer {
  let registry = WarpTestTextureRegistry()
  let factory = WarpTestAVFactory(pixels: pixels)
  return FVPTextureBasedVideoPlayer(
    playerItem: factory.item,
    frameUpdater: FVPFrameUpdater(registry: registry),
    displayLink: WarpTestDisplayLink(),
    avFactory: factory,
    viewProvider: WarpTestViewProvider()
  )
}

func makeWarpPixelBuffer() -> CVPixelBuffer {
  var buffer: CVPixelBuffer?
  CVPixelBufferCreate(nil, 1, 1, kCVPixelFormatType_32BGRA, nil, &buffer)
  return buffer!
}

#if os(iOS)
  final class WarpTestAudioSession: NSObject, FVPAVAudioSession {
    var category: AVAudioSession.Category = .ambient
    var categoryOptions: AVAudioSession.CategoryOptions = []
    func setCategory(
      _ category: AVAudioSession.Category,
      with options: AVAudioSession.CategoryOptions
    ) throws {
      self.category = category
      categoryOptions = options
    }
  }
#endif
