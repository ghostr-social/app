import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

void main() {
  test('close releases a ready video retained without a listener', () async {
    final gateway = _RetainedGateway();
    final port = AndroidIncomingVideoSharePort(gateway);
    final subscription = port.events.listen((_) {});
    await gateway.takeStarted.future;
    await subscription.cancel();
    gateway.takeResult.complete(const {
      'path': '/cache/shared/retained.mp4',
      'label': 'retained.mp4',
      'mimeType': 'video/mp4',
    });
    await Future<void>.delayed(Duration.zero);

    await port.close();

    expect(gateway.operations, ['release:/cache/shared/retained.mp4', 'close']);
  });
}

final class _RetainedGateway implements IncomingVideoShareGateway {
  final takeStarted = Completer<void>();
  final takeResult = Completer<Map<Object?, Object?>?>();
  final _available = StreamController<void>.broadcast();
  final operations = <String>[];

  @override
  Stream<void> get videoAvailable => _available.stream;

  @override
  Future<Map<Object?, Object?>?> takePendingVideo() {
    takeStarted.complete();
    return takeResult.future;
  }

  @override
  Future<void> acknowledgeVideo(String path) async {}

  @override
  Future<void> releaseVideo(String path) async {
    operations.add('release:$path');
  }

  @override
  Future<void> close() async {
    operations.add('close');
    await _available.close();
  }
}
