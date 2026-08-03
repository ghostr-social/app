import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/shared/widgets/profile_avatar.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('falls back to creator initials in the rail avatar',
      (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(creator: sampleCreator())],
    );

    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    expect(find.byTooltip('Open profile'), findsOneWidget);
    expect(
      find.descendant(
        of: find.byType(ProfileAvatar),
        matching: find.text('NR'),
      ),
      findsOneWidget,
    );
  });
}
