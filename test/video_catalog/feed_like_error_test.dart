import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('keeps the prior like state when publishing fails',
      (tester) async {
    final repository = FakeVideoCatalogRepository(
      forYouFeed: [samplePost()],
      writes: const FakeWriteScenario(
        likeFailure: AppFailure('No relay accepted the like'),
      ),
    );
    await tester.pumpWidget(feedScreenHarness(repository));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Like video'));
    await tester.pumpAndSettle();

    expect(find.text('No relay accepted the like'), findsOneWidget);
    expect(find.text('42'), findsOneWidget);
    expect(find.byTooltip('Like video'), findsOneWidget);
  });
}
