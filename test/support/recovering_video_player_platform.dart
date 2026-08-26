import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';
import 'video_player_test_events.dart';
final class RecoveringVideoPlayerPlatform extends VideoPlayerPlatform {
  RecoveringVideoPlayerPlatform({
    this.initializationFailures = 0,
    this.initializationErrorCode = 'initialize-failed',
    this.retainDisposedStreams = false,
  });

  final List<DataSource> dataSources = [];
  final List<String> commands = [];
  final Map<int, StreamController<VideoEvent>> _streams = {};
  Duration position = Duration.zero;
  int initializationFailures;
  final String initializationErrorCode;
  final bool retainDisposedStreams;
  int _nextId = 0;

  int get latestId => _nextId - 1;

  void failLatest(String message) {
    fail(latestId, message);
  }

  void fail(int id, String message) {
    _streams[id]!.addError(
      PlatformException(code: 'stream-failed', message: message),
    );
  }

  @override
  Future<int?> create(DataSource dataSource) async {
    final id = _nextId++;
    dataSources.add(dataSource);
    commands.add('create:$id');
    final stream = StreamController<VideoEvent>.broadcast();
    _streams[id] = stream;
    Timer.run(() => _initialize(stream));
    return id;
  }

  void _initialize(StreamController<VideoEvent> stream) {
    if (initializationFailures == 0) {
      stream.add(initializedVideoEvent);
      return;
    }
    initializationFailures -= 1;
    stream.addError(initializationError(initializationErrorCode));
  }

  @override
  Future<void> init() async {}

  @override
  Stream<VideoEvent> videoEventsFor(int textureId) {
    return _streams[textureId]!.stream;
  }

  @override
  Future<void> dispose(int textureId) async {
    commands.add('dispose:$textureId');
    if (!retainDisposedStreams) _streams.remove(textureId);
  }

  @override
  Future<void> play(int textureId) async {
    commands.add('play:$textureId');
  }

  @override
  Future<void> pause(int textureId) async {
    commands.add('pause:$textureId');
  }

  @override
  Future<void> seekTo(int textureId, Duration target) async {
    position = target;
    commands.add('seek:$textureId:${target.inMilliseconds}');
  }

  @override
  Future<Duration> getPosition(int textureId) async => position;

  @override
  Future<void> setLooping(int textureId, bool looping) async {
    commands.add('loop:$textureId');
  }

  @override
  Future<void> setVolume(int textureId, double volume) async {}

  @override
  Future<void> setPlaybackSpeed(int textureId, double speed) async {}

  @override
  Widget buildView(int textureId) => Texture(textureId: textureId);
}
