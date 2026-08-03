import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';

import '../support/compose_screen_harness.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('keeps the draft usable when video capture is unsupported',
      (tester) async {
    final picker = FakeMediaPickerPort(recoveredMedia: sampleMedia())
      ..capabilities =
          const MediaPickerCapabilities(library: true, camera: false);
    await tester.pumpWidget(composeScreenHarness(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));
    await tester.pumpAndSettle();

    final capture = tester.widget<FilledButton>(
      find.widgetWithText(FilledButton, 'Capture video'),
    );
    expect(capture.onPressed, isNull);
    expect(
      find.text('Video capture is unavailable on this device.'),
      findsOneWidget,
    );
    expect(
      find.bySemanticsLabel('Capture video unavailable on this device'),
      findsOneWidget,
    );
    expect(find.text('/tmp/ghostr-test.mp4'), findsOneWidget);
    expect(tester.widget<TextField>(find.byType(TextField)).enabled, isTrue);
    expect(picker.cameraPickCount, 0);
  });
}
