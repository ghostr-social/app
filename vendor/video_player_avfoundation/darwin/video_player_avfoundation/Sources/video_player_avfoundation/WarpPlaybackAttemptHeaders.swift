// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import Foundation

enum WarpPlaybackAttemptHeaderError: Error {
  case duplicate
  case malformed
}

struct WarpPlaybackAttemptHeaders {
  static let name = "X-Ghostr-Playback-Attempt"

  let httpHeaders: [String: String]
  let attemptToken: String?

  static func parse(_ headers: [String: String]) throws -> Self {
    let entries = headers.filter { key, _ in
      key.caseInsensitiveCompare(name) == .orderedSame
    }
    guard entries.count <= 1 else {
      throw WarpPlaybackAttemptHeaderError.duplicate
    }
    guard let entry = entries.first else {
      return Self(httpHeaders: headers, attemptToken: nil)
    }
    guard isCanonicalToken(entry.value) else {
      throw WarpPlaybackAttemptHeaderError.malformed
    }
    return Self(
      httpHeaders: headers.filter { $0.key != entry.key },
      attemptToken: entry.value
    )
  }

  private static func isCanonicalToken(_ token: String) -> Bool {
    guard token.count == 22 else { return false }
    let encoded =
      token.replacingOccurrences(of: "-", with: "+")
      .replacingOccurrences(of: "_", with: "/") + "=="
    guard let bytes = Data(base64Encoded: encoded), bytes.count == 16 else {
      return false
    }
    let canonical = bytes.base64EncodedString()
      .replacingOccurrences(of: "+", with: "-")
      .replacingOccurrences(of: "/", with: "_")
      .replacingOccurrences(of: "=", with: "")
    return canonical == token
  }
}
