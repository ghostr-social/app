import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('switching tabs preserves the create draft', (tester) async {
    final session = sampleSession();
    final dependencies = buildFakeDependencies(
      session: session,
      catalogRepository: FakeVideoCatalogRepository(forYouFeed: [samplePost()]),
      device: FakeDeviceDependencies(
        mediaPicker: FakeMediaPickerPort(galleryMedia: sampleMedia()),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('compose-caption-field')),
      'Keep this draft',
    );
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Create'));
    await tester.pumpAndSettle();

    expect(find.text('Keep this draft'), findsOneWidget);
    final publish = tester.widget<ElevatedButton>(
      find.widgetWithText(ElevatedButton, 'Publish'),
    );
    expect(publish.onPressed, isNotNull);
  });
}
