part of 'video_player_playback_port.dart';

enum _InitializationExit { initialized, closed, superseded }

Future<_InitializationExit> _waitForInitialization({
  required Future<void> initialization,
  required Future<void> closed,
  required Future<void> superseded,
}) {
  return Future.any([
    initialization.then((_) => _InitializationExit.initialized),
    closed.then((_) => _InitializationExit.closed),
    superseded.then((_) => _InitializationExit.superseded),
  ]);
}
