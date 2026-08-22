// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import Foundation

final class WarpFirstFrameReplayBuffer {
  private let capacity: Int
  private let lock = NSLock()
  private var history: [String] = []
  private var pending: [String] = []

  init(capacity: Int) {
    precondition(capacity > 0)
    self.capacity = capacity
  }

  @discardableResult
  func record(_ token: String) -> Bool {
    lock.lock()
    defer { lock.unlock() }
    guard !history.contains(token) else { return false }
    history.append(token)
    pending.append(token)
    trimToCapacity()
    return true
  }

  func cancel(_ token: String) {
    lock.lock()
    pending.removeAll { $0 == token }
    lock.unlock()
  }

  func deliverPending(_ deliver: (String) -> Void) {
    lock.lock()
    defer { lock.unlock() }
    let tokens = pending
    pending.removeAll(keepingCapacity: true)
    tokens.forEach(deliver)
  }

  private func trimToCapacity() {
    while history.count > capacity {
      let token = history.removeFirst()
      pending.removeAll { $0 == token }
    }
  }
}
