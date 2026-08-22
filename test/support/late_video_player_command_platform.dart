import 'dart:async';

import 'package:video_player_platform_interface/video_player_platform_interface.dart';

import 'audited_video_player_platform.dart';

enum LateVideoPlayerCommand { play, unmute }

final class LateVideoPlayerCommandPlatform extends AuditedVideoPlayerPlatform {
  final Map<int, DataSource> sources = {};
  Completer<void>? _gate;
  Completer<void>? _entered;
  LateVideoPlayerCommand? _blocked;

  void blockNext(LateVideoPlayerCommand command) {
    _blocked = command;
    _gate = Completer<void>();
    _entered = Completer<void>();
  }

  Future<void> get entered => _entered!.future;

  void release() {
    final gate = _gate;
    if (gate != null && !gate.isCompleted) gate.complete();
  }

  @override
  Future<int?> create(DataSource dataSource) async {
    final id = await super.create(dataSource);
    if (id != null) sources[id] = dataSource;
    return id;
  }

  @override
  Future<void> play(int textureId) async {
    await _waitIfBlocked(LateVideoPlayerCommand.play);
    await super.play(textureId);
  }

  @override
  Future<void> setVolume(int textureId, double volume) async {
    if (volume == 1) {
      await _waitIfBlocked(LateVideoPlayerCommand.unmute);
    }
    await super.setVolume(textureId, volume);
  }

  Future<void> _waitIfBlocked(LateVideoPlayerCommand command) async {
    if (_blocked != command) return;
    _blocked = null;
    _entered?.complete();
    await _gate!.future;
  }

  int playerFor(String url) {
    return sources.entries
        .singleWhere(
          (entry) => _matches(entry.value, url),
        )
        .key;
  }

  int latestPlayerFor(String url) {
    return sources.entries.lastWhere((entry) => _matches(entry.value, url)).key;
  }

  int creationsFor(String url) {
    return sources.values.where((source) => _matches(source, url)).length;
  }

  bool _matches(DataSource source, String url) {
    return source.uri == url || source.uri?.endsWith(url) == true;
  }
}
