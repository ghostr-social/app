import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_sharing.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows a download failure and restores the share action', (
    tester,
  ) async {
    final sharing = FakeVideoShareWorkflow(
      failure: const AppFailure('Could not download this video.'),
    );
    final repository = FakeVideoCatalogRepository(forYouFeed: [samplePost()]);
    await tester.pumpWidget(
      feedScreenHarness(repository, shareWorkflow: sharing),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Share video'));
    await tester.pumpAndSettle();

    expect(find.text('Could not download this video.'), findsOneWidget);
    expect(find.byTooltip('Share video'), findsOneWidget);
  });
}
