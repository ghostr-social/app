import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/compose_screen_harness.dart';

void main() {
  testWidgets('shows a permission error when video capture is denied',
      (tester) async {
    final picker = FakeMediaPickerPort(
      cameraFailure: const AppFailure('Camera access was denied.'),
    );
    await tester.pumpWidget(composeScreenHarness(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));

    await tester.tap(find.text('Capture video'));
    await tester.pump();

    expect(find.text('Camera access was denied.'), findsOneWidget);
  });
}
