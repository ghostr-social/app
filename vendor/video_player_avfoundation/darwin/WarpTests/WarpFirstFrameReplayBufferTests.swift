import Foundation
import Testing

@testable import WarpVideoPlayerCore

@Suite struct WarpFirstFrameReplayBufferTests {
  @Test func replaysUniqueSignalsOnlyOnce() {
    let buffer = WarpFirstFrameReplayBuffer(capacity: 8)
    #expect(buffer.record("AAAAAAAAAAAAAAAAAAAAAA"))
    #expect(!buffer.record("AAAAAAAAAAAAAAAAAAAAAA"))

    var delivered: [String] = []
    buffer.deliverPending { delivered.append($0) }
    buffer.deliverPending { delivered.append($0) }

    #expect(delivered == ["AAAAAAAAAAAAAAAAAAAAAA"])
  }

  @Test func boundsReplayAndDedupHistoryToEight() {
    let buffer = WarpFirstFrameReplayBuffer(capacity: 8)
    let tokens = (0..<9).map(canonicalToken)
    for token in tokens { #expect(buffer.record(token)) }

    var delivered: [String] = []
    buffer.deliverPending { delivered.append($0) }

    #expect(delivered == Array(tokens.suffix(8)))
    #expect(buffer.record(tokens[0]))
  }

  @Test func cancellationSuppressesPendingDeliveryButKeepsDedup() {
    let buffer = WarpFirstFrameReplayBuffer(capacity: 8)
    let token = canonicalToken(1)
    #expect(buffer.record(token))
    buffer.cancel(token)

    var delivered: [String] = []
    buffer.deliverPending { delivered.append($0) }

    #expect(delivered.isEmpty)
    #expect(!buffer.record(token))
  }
}

private func canonicalToken(_ value: Int) -> String {
  let suffix = String(format: "%02X", value)
  return String(repeating: "A", count: 20) + suffix
}
