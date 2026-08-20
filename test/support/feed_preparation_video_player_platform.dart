import 'dart:async';

import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'audited_video_player_platform.dart';

final class FeedPreparationVideoPlayerPlatform
    extends AuditedVideoPlayerPlatform {
  FeedPreparationVideoPlayerPlatform({super.autoInitialize});

  final Map<int, DataSource> sources = {};
  final Set<int> disposed = {};
  Completer<void>? _disposalGate;
  Completer<void>? _nextMuteGate;
  Completer<void>? _activeMuteGate;
  Completer<void>? _muteEntered;
  int? _muteFailure;
  var muteFailures = 0;
  var peakPlayerCount = 0;

  void blockDisposal() => _disposalGate = Completer<void>();

  void releaseDisposal() {
    final gate = _disposalGate;
    if (gate != null && !gate.isCompleted) gate.complete();
  }

  void failNextMute(int textureId) => _muteFailure = textureId;

  void blockNextMute() {
    _nextMuteGate = Completer<void>();
    _muteEntered = Completer<void>();
  }

  Future<void> get muteEntered => _muteEntered!.future;

  void releaseMute() {
    final gate = _activeMuteGate ?? _nextMuteGate;
    if (gate != null && !gate.isCompleted) gate.complete();
  }

  @override
  Future<int?> create(DataSource dataSource) async {
    final id = await super.create(dataSource);
    if (id == null) return null;
    sources[id] = dataSource;
    if (playerCount > peakPlayerCount) peakPlayerCount = playerCount;
    return id;
  }

  @override
  Future<void> dispose(int textureId) async {
    await _disposalGate?.future;
    disposed.add(textureId);
    await super.dispose(textureId);
  }

  @override
  Future<void> setVolume(int textureId, double volume) async {
    if (volume == 0 && _muteFailure == textureId) {
      _muteFailure = null;
      muteFailures += 1;
      throw StateError('mute failed');
    }
    final gate = volume == 0 ? _nextMuteGate : null;
    if (gate != null) {
      _nextMuteGate = null;
      _activeMuteGate = gate;
      _muteEntered?.complete();
      await gate.future;
      _activeMuteGate = null;
    }
    await super.setVolume(textureId, volume);
  }

  int playerFor(String url) {
    return sources.entries
        .singleWhere(
          (entry) =>
              entry.value.uri == url || entry.value.uri?.endsWith(url) == true,
        )
        .key;
  }

  int creationsFor(String url) {
    return sources.values.where((source) => source.uri == url).length;
  }
}
