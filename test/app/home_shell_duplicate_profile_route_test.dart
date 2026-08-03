import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_screen.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_screen.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('ignores a second profile request while one is opening',
      (tester) async {
    final post = samplePost();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [post],
        feed: FakeFeedScenario(
          profiles: {post.creator.id: sampleProfileDetails()},
        ),
      ),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    final feed = tester.widget<FeedScreen>(find.byType(FeedScreen));

    feed.bindings.onOpenProfile(post.creator.id);
    feed.bindings.onOpenProfile(post.creator.id);
    await tester.pumpAndSettle();
    expect(find.byType(ProfileScreen), findsOneWidget);

    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(find.byType(ProfileScreen), findsNothing);
  });
}
