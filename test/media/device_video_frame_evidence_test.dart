import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/support/device_video_frame_evidence.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'detects chromatic pixels and motion in the central video region',
    () async {
      final first = await _frame(20);
      final second = await _frame(60);

      final evidence = await DeviceVideoFrameEvidence.compare(first, second);

      expect(evidence.chromaticRatio, greaterThan(0.9));
      expect(evidence.changedRatio, greaterThan(0.1));
    },
  );

  test('rejects two static black video frames', () async {
    final black = await _frame(null);

    final evidence = await DeviceVideoFrameEvidence.compare(black, black);

    expect(evidence.chromaticRatio, 0);
    expect(evidence.changedRatio, 0);
  });

  test('rejects playback that turns black after a colored frame', () async {
    final colored = await _frame(20);
    final black = await _frame(null);

    final evidence = await DeviceVideoFrameEvidence.compare(colored, black);

    expect(evidence.chromaticRatio, 0);
  });
}

Future<List<int>> _frame(double? bandTop) async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  canvas.drawRect(
    const Rect.fromLTWH(0, 0, 100, 100),
    Paint()..color = bandTop == null ? Colors.black : Colors.blue,
  );
  if (bandTop != null) {
    canvas.drawRect(
      Rect.fromLTWH(0, bandTop, 100, 20),
      Paint()..color = Colors.yellow,
    );
  }
  final image = await recorder.endRecording().toImage(100, 100);
  final data = await image.toByteData(format: ui.ImageByteFormat.png);
  image.dispose();
  return data!.buffer.asUint8List();
}
