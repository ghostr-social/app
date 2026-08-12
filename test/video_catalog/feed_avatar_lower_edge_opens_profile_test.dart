import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('the lower avatar edge opens the profile instead of following', (
    tester,
  ) async {
    final creator = sampleCreator();
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: creator)],
    );
    final openedProfiles = <String>[];
    await tester.pumpWidget(
      feedScreenHarness(
        repository,
        options: FeedScreenHarnessOptions(onOpenProfile: openedProfiles.add),
      ),
    );
    await tester.pumpAndSettle();
    final avatar = find.byTooltip('Open profile');
    final avatarRect = tester.getRect(avatar);

    await tester.tapAt(avatarRect.bottomCenter.translate(0, -2));
    await tester.pumpAndSettle();

    expect(openedProfiles, [creator.id.value]);
    expect(repository.followedProfiles, isNot(contains(creator.id)));
  });
}
