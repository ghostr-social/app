import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

class FakeVideoPlayerPlatform extends VideoPlayerPlatform {
  FakeVideoPlayerPlatform({this.autoInitialize = true});

  final bool autoInitialize;
  final List<String> calls = [];
  final List<DataSource> dataSources = [];
  final Map<int, StreamController<VideoEvent>> _streams = {};
  final Set<String> failingCalls = {};
  final Completer<void> disposeStarted = Completer<void>();
  Completer<void>? disposeBarrier;
  bool failNextInitialization = false;
  int _nextTextureId = 0;

  @override
  Future<int?> create(DataSource dataSource) async {
    calls.add('create');
    dataSources.add(dataSource);
    final id = _nextTextureId++;
    final stream = StreamController<VideoEvent>.broadcast(
      onCancel: () => calls.add('cancel'),
    );
    _streams[id] = stream;
    if (autoInitialize) Timer.run(() => _initialize(stream));
    return id;
  }

  void _initialize(StreamController<VideoEvent> stream) {
    if (failNextInitialization) {
      failNextInitialization = false;
      stream.addError(
        PlatformException(code: 'video-error', message: 'Cannot load video'),
      );
      return;
    }
    stream.add(
      VideoEvent(
        eventType: VideoEventType.initialized,
        size: const Size(180, 320),
        duration: const Duration(seconds: 10),
      ),
    );
  }

  @override
  Future<void> init() async => calls.add('init');

  @override
  Stream<VideoEvent> videoEventsFor(int textureId) {
    return _streams[textureId]!.stream;
  }

  @override
  Future<void> dispose(int textureId) async {
    calls.add('dispose');
    if (!disposeStarted.isCompleted) disposeStarted.complete();
    await disposeBarrier?.future;
    _streams.remove(textureId);
  }

  @override
  Future<void> play(int textureId) async => _recordPlaybackCall('play');

  @override
  Future<void> pause(int textureId) async => _recordPlaybackCall('pause');

  @override
  Future<void> seekTo(int textureId, Duration position) async {
    _recordPlaybackCall('seekTo');
  }

  @override
  Future<Duration> getPosition(int textureId) async => Duration.zero;

  @override
  Future<void> setLooping(int textureId, bool looping) async {
    calls.add('setLooping');
  }

  @override
  Future<void> setVolume(int textureId, double volume) async {}

  @override
  Future<void> setPlaybackSpeed(int textureId, double speed) async {}

  @override
  Widget buildView(int textureId) => Texture(textureId: textureId);

  void _recordPlaybackCall(String call) {
    calls.add(call);
    if (failingCalls.contains(call)) {
      throw PlatformException(code: '$call-failed');
    }
  }
}
