import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/compose_screen_harness.dart';
import '../support/fakes.dart';
import '../support/pending_video_publishing_repository.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('disables composer inputs while publishing', (tester) async {
    final publishing = PendingVideoPublishingRepository();
    await tester.pumpWidget(composeScreenHarness(
      publishing: publishing,
      activity: FakeActivityRepository(),
      picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
    ));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    await tester.pump();

    expect(_elevated(tester, 'Choose from library').onPressed, isNull);
    expect(_filled(tester, 'Capture video').onPressed, isNull);
    expect(tester.widget<TextField>(find.byType(TextField)).enabled, isFalse);
    expect(_elevated(tester, 'Publishing...').onPressed, isNull);

    publishing.result.complete(samplePost());
    await tester.pumpAndSettle();
  });
}

ElevatedButton _elevated(WidgetTester tester, String label) {
  return tester.widget(find.widgetWithText(ElevatedButton, label));
}

FilledButton _filled(WidgetTester tester, String label) {
  return tester.widget(find.widgetWithText(FilledButton, label));
}
