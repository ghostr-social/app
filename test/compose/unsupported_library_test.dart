import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/media_picker_capabilities.dart';

import '../support/compose_screen_harness.dart';
import '../support/fakes.dart';

void main() {
  testWidgets('disables browser library selection instead of using blob paths',
      (tester) async {
    final picker = FakeMediaPickerPort()
      ..capabilities = const MediaPickerCapabilities.noneSupported();
    await tester.pumpWidget(composeScreenHarness(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));
    await tester.pumpAndSettle();

    final choose = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Choose from library'),
    );
    expect(choose.onPressed, isNull);
    expect(
      find.text('Video library selection is unavailable on this device.'),
      findsOneWidget,
    );
    expect(
      find.bySemanticsLabel('Video library unavailable on this device'),
      findsOneWidget,
    );
    expect(picker.galleryPickCount, 0);
  });
}
