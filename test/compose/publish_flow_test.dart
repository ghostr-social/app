import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/compose_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('publishes a selected video from the composer', (tester) async {
    final catalogRepository =
        FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    final activityRepository = FakeActivityRepository();
    final mediaPickerPort = FakeMediaPickerPort(galleryMedia: sampleMedia());

    await tester.pumpWidget(composeScreenHarness(
      publishing: catalogRepository,
      activity: activityRepository,
      picker: mediaPickerPort,
    ));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose from library'));
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('Publish'));
    await tester.tap(find.text('Publish'));
    await tester.pumpAndSettle();

    expect(find.text('Published to your Ghostr profile.'), findsOneWidget);
    expect(catalogRepository.forYouFeed.first.caption, 'ghostr-test.mp4');
  });
}
