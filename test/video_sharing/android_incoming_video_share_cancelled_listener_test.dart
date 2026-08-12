import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

void main() {
  test('replays an in-flight result after its listener cancels', () async {
    final gateway = _PendingIncomingVideoGateway();
    addTearDown(gateway.close);
    final port = AndroidIncomingVideoSharePort(gateway);
    final firstEvents = <IncomingVideoShareEvent>[];
    final firstSubscription = port.events.listen(firstEvents.add);
    await gateway.takeStarted.future;

    await firstSubscription.cancel();
    gateway.takeResult.complete(const {
      'path': '/cache/shared/retained.mp4',
      'label': 'retained.mp4',
      'mimeType': 'video/mp4',
    });
    await Future<void>.delayed(Duration.zero);

    final replayed = await port.events.first.timeout(
      const Duration(milliseconds: 500),
    );
    expect(firstEvents, isEmpty);
    expect((replayed as IncomingVideoShareReady).media.label, 'retained.mp4');
    expect(gateway.takeCalls, 1);
  });
}

final class _PendingIncomingVideoGateway implements IncomingVideoShareGateway {
  final takeStarted = Completer<void>();
  final takeResult = Completer<Map<Object?, Object?>?>();
  final _videoAvailable = StreamController<void>.broadcast();
  int takeCalls = 0;

  @override
  Stream<void> get videoAvailable => _videoAvailable.stream;

  @override
  Future<void> acknowledgeVideo(String path) async {}

  @override
  Future<void> releaseVideo(String path) async {}

  @override
  Future<Map<Object?, Object?>?> takePendingVideo() {
    takeCalls += 1;
    if (!takeStarted.isCompleted) takeStarted.complete();
    return takeCalls == 1 ? takeResult.future : Future.value();
  }

  @override
  Future<void> close() => _videoAvailable.close();
}
