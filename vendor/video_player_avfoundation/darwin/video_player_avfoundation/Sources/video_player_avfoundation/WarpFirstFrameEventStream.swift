// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

import Foundation

#if os(iOS)
  import Flutter
#else
  import FlutterMacOS
#endif

final class WarpFirstFrameEventStream: NSObject, FlutterStreamHandler {
  static let channelName = "social.ghostr/video_player_first_frames"

  private let channel: FlutterEventChannel
  private let replay = WarpFirstFrameReplayBuffer(capacity: 8)
  private let lock = NSLock()
  private var eventSink: FlutterEventSink?
  private var deliveryScheduled = false

  init(binaryMessenger: FlutterBinaryMessenger) {
    channel = FlutterEventChannel(
      name: Self.channelName,
      binaryMessenger: binaryMessenger
    )
    super.init()
    channel.setStreamHandler(self)
  }

  func report(_ token: String) {
    guard replay.record(token) else { return }
    scheduleDelivery()
  }

  func cancel(_ token: String) {
    replay.cancel(token)
  }

  func detach() {
    channel.setStreamHandler(nil)
    lock.lock()
    eventSink = nil
    lock.unlock()
  }

  func onListen(
    withArguments arguments: Any?,
    eventSink events: @escaping FlutterEventSink
  ) -> FlutterError? {
    lock.lock()
    eventSink = events
    lock.unlock()
    scheduleDelivery()
    return nil
  }

  func onCancel(withArguments arguments: Any?) -> FlutterError? {
    lock.lock()
    eventSink = nil
    lock.unlock()
    return nil
  }

  private func scheduleDelivery() {
    lock.lock()
    guard !deliveryScheduled else {
      lock.unlock()
      return
    }
    deliveryScheduled = true
    lock.unlock()
    DispatchQueue.main.async { [weak self] in self?.deliver() }
  }

  private func deliver() {
    dispatchPrecondition(condition: .onQueue(.main))
    lock.lock()
    deliveryScheduled = false
    let sink = eventSink
    lock.unlock()
    guard let sink else { return }
    replay.deliverPending { token in
      sink(["version": 1, "attemptToken": token])
    }
  }
}
