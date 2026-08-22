import Foundation
import Testing

@testable import video_player_avfoundation

private struct CapturedFirstFrameEvent {
  let value: Any?
  let deliveredOnMainThread: Bool
}

@Suite struct WarpFirstFrameEventStreamTests {
  @Test func replaysAnEarlyFrameOnTheMainThread() async {
    let stream = WarpFirstFrameEventStream(binaryMessenger: WarpTestMessenger())
    let token = "AAAAAAAAAAAAAAAAAAAAAA"
    stream.report(token)

    let event: CapturedFirstFrameEvent = await withCheckedContinuation { continuation in
      _ = stream.onListen(withArguments: nil) { value in
        continuation.resume(
          returning: CapturedFirstFrameEvent(
            value: value,
            deliveredOnMainThread: Thread.isMainThread
          ))
      }
    }

    let payload = event.value as? [String: Any]
    let version = payload?["version"] as? Int
    let deliveredToken = payload?["attemptToken"] as? String
    #expect(version == 1)
    #expect(deliveredToken == token)
    #expect(event.deliveredOnMainThread)
  }
}
