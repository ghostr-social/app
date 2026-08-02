import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/compose_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows an upload error and re-enables publishing',
      (tester) async {
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      writes: const FakeWriteScenario(
        publishFailure: AppFailure('Blossom rejected the upload'),
      ),
    );
    await tester.pumpWidget(composeScreenHarness(
      publishing: catalog,
      activity: FakeActivityRepository(),
      picker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
    ));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    await tester.pumpAndSettle();

    expect(find.text('Blossom rejected the upload'), findsOneWidget);
    expect(find.widgetWithText(ElevatedButton, 'Publish'), findsOneWidget);
  });
}
