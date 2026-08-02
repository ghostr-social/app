import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/compose_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('restores a video if Android recreated the picker activity',
      (tester) async {
    final picker = FakeMediaPickerPort(recoveredMedia: sampleMedia());
    await tester.pumpWidget(composeScreenHarness(
      publishing: FakeVideoCatalogRepository(forYouFeed: []),
      activity: FakeActivityRepository(),
      picker: picker,
    ));
    await tester.pumpAndSettle();

    expect(find.text(sampleMedia().path), findsOneWidget);
    expect(find.text('No draft selected'), findsNothing);
  });
}
