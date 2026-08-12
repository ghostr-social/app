import 'dart:async';

import 'package:flutter/services.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

final class MethodChannelIncomingVideoShareGateway
    implements IncomingVideoShareGateway {
  MethodChannelIncomingVideoShareGateway([
    this._channel = const MethodChannel('app.ghostr/incoming_video_share'),
  ]) {
    _channel.setMethodCallHandler(_handleMethodCall);
  }

  final MethodChannel _channel;
  final _videoAvailable = StreamController<void>.broadcast();
  Future<void>? _closeFuture;

  @override
  Stream<void> get videoAvailable => _videoAvailable.stream;

  @override
  Future<Map<Object?, Object?>?> takePendingVideo() {
    return _channel.invokeMapMethod<Object?, Object?>('takePendingVideo');
  }

  @override
  Future<void> acknowledgeVideo(String path) {
    return _channel.invokeMethod<void>('acknowledgeVideo', path);
  }

  @override
  Future<void> releaseVideo(String path) {
    return _channel.invokeMethod<void>('releaseVideo', path);
  }

  @override
  Future<void> close() => _closeFuture ??= _close();

  Future<void> _close() async {
    _channel.setMethodCallHandler(null);
    await _videoAvailable.close();
  }

  Future<void> _handleMethodCall(MethodCall call) async {
    if (call.method != 'videoAvailable' || _videoAvailable.isClosed) return;
    _videoAvailable.add(null);
  }
}
