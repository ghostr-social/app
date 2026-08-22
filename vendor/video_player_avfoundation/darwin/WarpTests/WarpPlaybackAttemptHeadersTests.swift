import Testing

@testable import WarpVideoPlayerCore

private let header = "X-Ghostr-Playback-Attempt"
private let token = "AAAAAAAAAAAAAAAAAAAAAA"

@Suite struct WarpPlaybackAttemptHeadersTests {
  @Test func extractsAndStripsOneCanonicalToken() throws {
    let parsed = try WarpPlaybackAttemptHeaders.parse([
      "Authorization": "Bearer normal",
      header: token,
    ])

    #expect(parsed.attemptToken == token)
    #expect(parsed.httpHeaders == ["Authorization": "Bearer normal"])
  }

  @Test func preservesOrdinaryHeadersWithoutAToken() throws {
    let headers = ["Range": "bytes=0-99"]
    let parsed = try WarpPlaybackAttemptHeaders.parse(headers)

    #expect(parsed.attemptToken == nil)
    #expect(parsed.httpHeaders == headers)
  }

  @Test func rejectsCaseInsensitiveDuplicates() {
    let headers = [header: token, header.lowercased(): token]
    #expect(throws: WarpPlaybackAttemptHeaderError.self) {
      try WarpPlaybackAttemptHeaders.parse(headers)
    }
  }

  @Test(arguments: ["short", "AAAAAAAAAAAAAAAAAAAAA!", "AAAAAAAAAAAAAAAAAAAAAB"])
  func rejectsMalformedOrNonCanonicalTokens(_ value: String) {
    #expect(throws: WarpPlaybackAttemptHeaderError.self) {
      try WarpPlaybackAttemptHeaders.parse([header: value])
    }
  }
}
