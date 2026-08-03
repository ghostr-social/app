import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('opens the creator profile when the rail avatar is tapped',
      (tester) async {
    final openedProfiles = <String>[];
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
    );

    await tester.pumpWidget(feedScreenHarness(
      repository,
      onOpenProfile: openedProfiles.add,
    ));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open profile'));
    await tester.pump();

    expect(openedProfiles, ['creator-1']);
  });
}
