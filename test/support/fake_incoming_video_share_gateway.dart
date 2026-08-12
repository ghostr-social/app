import 'dart:async';
import 'dart:collection';

import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

final class FakeIncomingVideoShareGateway implements IncomingVideoShareGateway {
  FakeIncomingVideoShareGateway({
    Iterable<Map<Object?, Object?>?> pendingVideos = const [],
    this.takeFailure,
  }) : _pendingVideos = Queue.of(pendingVideos);

  final Queue<Map<Object?, Object?>?> _pendingVideos;
  final _videoAvailable = StreamController<void>.broadcast();
  final _firstTake = Completer<void>();
  final Object? takeFailure;
  int takePendingVideoCalls = 0;
  int closeCalls = 0;
  final acknowledgedPaths = <String>[];
  final releasedPaths = <String>[];

  Future<void> get firstTake => _firstTake.future;

  @override
  Stream<void> get videoAvailable => _videoAvailable.stream;

  @override
  Future<Map<Object?, Object?>?> takePendingVideo() async {
    takePendingVideoCalls += 1;
    if (!_firstTake.isCompleted) _firstTake.complete();
    final failure = takeFailure;
    if (failure != null) throw failure;
    return _pendingVideos.isEmpty ? null : _pendingVideos.removeFirst();
  }

  @override
  Future<void> acknowledgeVideo(String path) async {
    acknowledgedPaths.add(path);
  }

  @override
  Future<void> releaseVideo(String path) async {
    releasedPaths.add(path);
  }

  void addPendingVideo(Map<Object?, Object?> payload) {
    _pendingVideos.add(payload);
  }

  void notifyVideoAvailable() => _videoAvailable.add(null);

  bool get hasVideoAvailableListener => _videoAvailable.hasListener;

  @override
  Future<void> close() async {
    closeCalls += 1;
    if (!_videoAvailable.isClosed) await _videoAvailable.close();
  }
}
