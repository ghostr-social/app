import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/widgets/feed_card.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('long-pressing a video offers blocking its creator',
      (tester) async {
    final spammer = sampleCreator(id: 'creator-spam', displayName: 'Spam');
    final kept = sampleCreator(id: 'creator-kept', displayName: 'Nora');
    final repository = FakeVideoCatalogRepository(forYouFeed: [
      samplePost(id: 'post-1', caption: 'Blocked clip', creator: spammer),
      samplePost(id: 'post-2', caption: 'Kept clip', creator: kept),
    ]);
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.longPress(find.byType(FeedCard));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Block ${spammer.handle}'));
    await tester.pumpAndSettle();

    expect(repository.blockedProfiles, contains(spammer.id));
    expect(find.text('Blocked clip'), findsNothing);
    expect(find.text('Kept clip'), findsOneWidget);
  });
}
