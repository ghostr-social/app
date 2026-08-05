import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

/// A player platform that initializes immediately and lets the test
/// script later value events (buffering, errors) onto the stream.
class ScriptedVideoPlayerPlatform extends VideoPlayerPlatform {
  ScriptedVideoPlayerPlatform({this.initializedSize = const Size(180, 320)});

  final Size initializedSize;
  final List<DataSource> dataSources = [];
  final Map<int, StreamController<VideoEvent>> _streams = {};
  int playCalls = 0;
  int _nextTextureId = 0;

  void emit(VideoEvent event) => _latestStream.add(event);

  void emitError(String message) {
    _latestStream.addError(
      PlatformException(code: 'video-error', message: message),
    );
  }

  StreamController<VideoEvent> get _latestStream =>
      _streams[_nextTextureId - 1]!;

  @override
  Future<int?> create(DataSource dataSource) async {
    dataSources.add(dataSource);
    final id = _nextTextureId++;
    final stream = StreamController<VideoEvent>.broadcast();
    _streams[id] = stream;
    Timer.run(() {
      stream.add(
        VideoEvent(
          eventType: VideoEventType.initialized,
          size: initializedSize,
          duration: const Duration(seconds: 10),
        ),
      );
    });
    return id;
  }

  @override
  Future<void> init() async {}

  @override
  Stream<VideoEvent> videoEventsFor(int textureId) {
    return _streams[textureId]!.stream;
  }

  @override
  Future<void> dispose(int textureId) async {}

  @override
  Future<void> play(int textureId) async {
    playCalls += 1;
  }

  @override
  Future<void> pause(int textureId) async {}

  @override
  Future<void> seekTo(int textureId, Duration position) async {}

  @override
  Future<Duration> getPosition(int textureId) async => Duration.zero;

  @override
  Future<void> setLooping(int textureId, bool looping) async {}

  @override
  Future<void> setVolume(int textureId, double volume) async {}

  @override
  Future<void> setPlaybackSpeed(int textureId, double speed) async {}

  @override
  Widget buildView(int textureId) => Texture(textureId: textureId);
}
