import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets(
    'the lower avatar edge beside the follow badge opens the profile',
    (tester) async {
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
      final avatarRect = tester.getRect(find.byTooltip('Open profile'));
      final followRect = tester.getRect(
        find.byTooltip('Follow ${creator.displayName}'),
      );

      // The follow badge straddles the avatar's bottom edge and owns the
      // taps inside its circle; the avatar's lower rim beside it must still
      // open the profile rather than follow.
      await tester.tapAt(Offset(followRect.left - 2, avatarRect.bottom - 6));
      await tester.pumpAndSettle();

      expect(openedProfiles, [creator.id.value]);
      expect(repository.followedProfiles, isNot(contains(creator.id)));
    },
  );
}
