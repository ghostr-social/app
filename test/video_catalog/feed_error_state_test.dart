import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('shows the feed error state and retry action', (tester) async {
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [],
        feed: const FakeFeedScenario(
          failure: AppFailure('Failed to load feed'),
        ),
      ),
    );

    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();

    expect(find.text('Failed to load feed'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });
}
