import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/incoming_video_share.dart';
import 'package:ghostr/platform/sharing/android_incoming_video_share_port.dart';

void main() {
  test('close releases a copied video returned by an in-flight take', () async {
    final gateway = _InFlightGateway();
    final port = AndroidIncomingVideoSharePort(gateway);
    final events = <IncomingVideoShareEvent>[];
    port.events.listen(events.add);
    await gateway.takeStarted.future;

    final closing = port.close();
    gateway.takeResult.complete(const {
      'path': '/cache/shared/in-flight.mp4',
      'label': 'in-flight.mp4',
      'mimeType': 'video/mp4',
    });
    await closing;

    expect(events, isEmpty);
    expect(gateway.operations, [
      'release:/cache/shared/in-flight.mp4',
      'close',
    ]);
  });
}

final class _InFlightGateway implements IncomingVideoShareGateway {
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
