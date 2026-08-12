import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/social/data/social_graph_cache.dart';
import 'package:ghostr/features/video_catalog/data/local_video_store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('cached follows suppress the feed action while offline', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    SharedPreferences.setMockInitialValues({});
    final creator = sampleCreator(id: 'cached-creator');
    final local = LocalVideoStore(
      await SharedPreferences.getInstance(),
      accountScope: testAccountStorageScope(),
    );
    await local.saveFollowedProfiles({creator.id});
    final remote = FakeNostrSocialPort()
      ..loadFailure = const AppFailure('Relay offline.');
    final social = SocialGraphCache(remote, local, RecordingFailureReporter());
    final feed = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
    );

    await tester.pumpWidget(
      feedScreenHarness(
        feed,
        options: FeedScreenHarnessOptions(social: social),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byTooltip('Open profile'), findsOneWidget);
    expect(
      find.bySemanticsLabel('Follow ${creator.displayName}'),
      findsNothing,
    );
    semantics.dispose();
  });
}
