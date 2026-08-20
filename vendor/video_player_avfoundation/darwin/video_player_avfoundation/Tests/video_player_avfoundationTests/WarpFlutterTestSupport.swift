import video_player_avfoundation_objc

@testable import video_player_avfoundation

#if os(iOS)
  import Flutter
  import UIKit
#else
  import FlutterMacOS
#endif

final class WarpTestMessenger: NSObject, FlutterBinaryMessenger {
  func send(onChannel channel: String, message: Data?) {}
  func send(
    onChannel channel: String,
    message: Data?,
    binaryReply callback: FlutterBinaryReply? = nil
  ) {}
  func setMessageHandlerOnChannel(
    _ channel: String,
    binaryMessageHandler handler: FlutterBinaryMessageHandler? = nil
  ) -> FlutterBinaryMessengerConnection { 0 }
  func cleanUpConnection(_ connection: FlutterBinaryMessengerConnection) {}
}

final class WarpTestTextureRegistry: NSObject, FlutterTextureRegistry {
  var onRegister: ((FlutterTexture) -> Void)?
  func register(_ texture: FlutterTexture) -> Int64 {
    onRegister?(texture)
    return 1
  }
  func unregisterTexture(_ textureId: Int64) {}
  func textureFrameAvailable(_ textureId: Int64) {}
}

final class WarpTestDisplayLink: NSObject, FVPDisplayLink {
  var running = false
  var duration: CFTimeInterval { 1.0 / 60.0 }
}

final class WarpTestDisplayLinkFactory: DisplayLinkFactory {
  let link = WarpTestDisplayLink()
  func displayLink(
    with viewProvider: FVPViewProvider,
    callback: @escaping () -> Void
  ) -> FVPDisplayLink { link }
}

final class WarpTestViewProvider: NSObject, FVPViewProvider {
  #if os(iOS)
    var viewController: UIViewController? = UIViewController()
  #else
    var view: NSView? = {
      let view = NSView()
      view.wantsLayer = true
      return view
    }()
  #endif
}

final class WarpTestAssetProvider: NSObject, FVPAssetProvider {
  func lookupKey(forAsset asset: String) -> String? { asset }
  func lookupKey(forAsset asset: String, fromPackage package: String) -> String? { asset }
}

func makeWarpTestPlugin(
  factory: WarpTestAVFactory,
  registry: WarpTestTextureRegistry = WarpTestTextureRegistry()
) -> VideoPlayerPlugin {
  VideoPlayerPlugin(
    avFactory: factory,
    displayLinkFactory: WarpTestDisplayLinkFactory(),
    binaryMessenger: WarpTestMessenger(),
    textureRegistry: registry,
    viewProvider: WarpTestViewProvider(),
    assetProvider: WarpTestAssetProvider()
  )
}
