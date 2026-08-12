import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

void main() {
  test('retains a replay when its next listener cancels immediately', () async {
    final gateway = _DelayedGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    final first = port.events.listen((_) {});
    await gateway.started.future;
    await first.cancel();
    gateway.result.complete(const {
      'path': '/cache/shared/replayed.mp4',
      'label': 'replayed.mp4',
      'mimeType': 'video/mp4',
    });
    await Future<void>.delayed(Duration.zero);

    final delivered = <IncomingVideoShareEvent>[];
    final cancelledReplay = port.events.listen(delivered.add);
    scheduleMicrotask(cancelledReplay.cancel);
    await Future<void>.delayed(Duration.zero);
    final event = delivered.isNotEmpty
        ? delivered.single
        : await port.events.first.timeout(const Duration(milliseconds: 500));

    expect((event as IncomingVideoShareReady).media.label, 'replayed.mp4');
  });
}

final class _DelayedGateway implements IncomingVideoShareGateway {
  final started = Completer<void>();
  final result = Completer<Map<Object?, Object?>?>();
  final _available = StreamController<void>.broadcast();

  @override
  Stream<void> get videoAvailable => _available.stream;

  @override
  Future<void> acknowledgeVideo(String path) async {}

  @override
  Future<void> releaseVideo(String path) async {}

  @override
  Future<Map<Object?, Object?>?> takePendingVideo() {
    if (!started.isCompleted) started.complete();
    return result.future;
  }

  @override
  Future<void> close() => _available.close();
}
