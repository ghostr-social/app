import 'dart:async';

import 'package:flutter/material.dart';
import 'package:video_player_platform_interface/video_player_platform_interface.dart';

final class AuditedVideoPlayerPlatform extends VideoPlayerPlatform {
  final Map<int, StreamController<VideoEvent>> _streams = {};
  final Map<int, _PlayerState> _states = {};
  final List<String> commands = [];
  Completer<void>? pauseGate;
  var _nextId = 0;
  var audibleOverlap = false;

  int get playerCount => _states.length;

  @override
  Future<int?> create(DataSource dataSource) async {
    final id = _nextId++;
    final stream = StreamController<VideoEvent>.broadcast();
    _streams[id] = stream;
    _states[id] = _PlayerState();
    Timer.run(() => stream.add(_initialized));
    return id;
  }

  @override
  Future<void> init() async {}

  @override
  Stream<VideoEvent> videoEventsFor(int textureId) =>
      _streams[textureId]!.stream;

  @override
  Future<void> dispose(int textureId) async {
    _states.remove(textureId);
    await _streams.remove(textureId)?.close();
  }

  @override
  Future<void> play(int textureId) async {
    commands.add('play:$textureId');
    _states[textureId]!.isPlaying = true;
    _audit();
  }

  @override
  Future<void> pause(int textureId) async {
    commands.add('pause:$textureId');
    await pauseGate?.future;
    _states[textureId]!.isPlaying = false;
    _audit();
  }

  @override
  Future<void> seekTo(int textureId, Duration position) async {}

  @override
  Future<Duration> getPosition(int textureId) async => Duration.zero;

  @override
  Future<void> setLooping(int textureId, bool looping) async {}

  @override
  Future<void> setVolume(int textureId, double volume) async {
    commands.add('volume:$textureId:$volume');
    _states[textureId]!.volume = volume;
    _audit();
  }

  @override
  Future<void> setPlaybackSpeed(int textureId, double speed) async {}

  @override
  Widget buildView(int textureId) => Texture(textureId: textureId);

  void _audit() {
    final audible = _states.values.where(
      (state) => state.isPlaying && state.volume > 0,
    );
    if (audible.length > 1) audibleOverlap = true;
  }
}

final class _PlayerState {
  var isPlaying = false;
  var volume = 1.0;
}

final _initialized = VideoEvent(
  eventType: VideoEventType.initialized,
  size: const Size(180, 320),
  duration: const Duration(seconds: 10),
);
