import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('shows a retryable comments error', (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      comments: const FakeCommentsScenario(
        failure: AppFailure('Relay timed out'),
      ),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Open comments'));
    await tester.pumpAndSettle();
    expect(find.text('Relay timed out'), findsOneWidget);

    repository.commentsFailure = null;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();

    expect(find.text('No comments yet'), findsOneWidget);
    expect(repository.commentLoadCount, 2);
  });
}
