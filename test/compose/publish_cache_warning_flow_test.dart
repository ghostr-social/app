import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/publish/domain/video_publication.dart';

import '../support/compose_screen_harness.dart';
import '../support/fakes.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows degraded success when the local catalog write fails',
      (tester) async {
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [],
      cacheStatus: VideoPublicationCacheStatus.unavailable,
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

    expect(
      find.text('Published, but your local video list could not be updated.'),
      findsOneWidget,
    );
    expect(catalog.forYouFeed, hasLength(1));
  });
}
