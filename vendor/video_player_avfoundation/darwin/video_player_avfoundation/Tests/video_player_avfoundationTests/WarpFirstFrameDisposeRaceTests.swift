import Foundation
import Testing

@testable import video_player_avfoundation

#if os(iOS)
  import Flutter
#else
  import FlutterMacOS
#endif

private final class WarpSendableBox<Value>: @unchecked Sendable {
  let value: Value
  init(_ value: Value) { self.value = value }
}

@MainActor @Suite struct WarpFirstFrameDisposeRaceTests {
  @Test func disposeWaitsForAnInFlightFirstFrameCallback() {
    let pixels = WarpTestPixelBufferSource()
    let player = makeWarpTexturePlayer(pixels: pixels)
    let callbackStarted = DispatchSemaphore(value: 0)
    let allowCallbackToFinish = DispatchSemaphore(value: 0)
    let copyFinished = DispatchSemaphore(value: 0)
    let disposeStarted = DispatchSemaphore(value: 0)
    let disposeFinished = DispatchSemaphore(value: 0)

    player.warpFirstFrameRenderedCallback = {
      callbackStarted.signal()
      allowCallbackToFinish.wait()
    }
    pixels.pixelBuffer = makeWarpPixelBuffer()
    let box = WarpSendableBox(player)
    DispatchQueue.global().async {
      box.value.copyPixelBuffer()
      copyFinished.signal()
    }
    #expect(callbackStarted.wait(timeout: .now() + 1) == .success)

    DispatchQueue.global().async {
      disposeStarted.signal()
      var error: FlutterError?
      box.value.disposeWithError(&error)
      disposeFinished.signal()
    }
    #expect(disposeStarted.wait(timeout: .now() + 1) == .success)
    #expect(disposeFinished.wait(timeout: .now() + 0.05) == .timedOut)

    allowCallbackToFinish.signal()
    #expect(copyFinished.wait(timeout: .now() + 1) == .success)
    #expect(disposeFinished.wait(timeout: .now() + 1) == .success)
  }
}
