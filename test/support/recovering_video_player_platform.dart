import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

final class RecoveringVideoPlayerPlatform extends VideoPlayerPlatform {
  RecoveringVideoPlayerPlatform({this.initializationFailures = 0});

  final List<DataSource> dataSources = [];
  final List<String> commands = [];
  final Map<int, StreamController<VideoEvent>> _streams = {};
  Duration position = Duration.zero;
  int initializationFailures;
  int _nextId = 0;

  int get latestId => _nextId - 1;

  void failLatest(String message) {
    _streams[latestId]!.addError(
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
      stream.add(_initialized);
      return;
    }
    initializationFailures -= 1;
    stream.addError(
      PlatformException(code: 'initialize-failed', message: 'Source failed'),
    );
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
    _streams.remove(textureId);
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

final _initialized = VideoEvent(
  eventType: VideoEventType.initialized,
  size: Size(180, 320),
  duration: Duration(seconds: 30),
);
