import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';

import '../support/fakes.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('returning to Search hides a video watched while away', (
    tester,
  ) async {
    final stale = samplePost(id: 'stale', caption: 'Already watched clip');
    final fresh = samplePost(id: 'fresh', caption: 'Fresh clip');
    final history = FakeWatchHistoryRepository();
    final catalog = FakeVideoCatalogRepository(
      forYouFeed: [samplePost(id: 'home')],
      feed: FakeFeedScenario(searchResults: [stale, fresh]),
    );
    await tester.pumpWidget(
      buildTestApp(
        buildFakeDependencies(
          session: sampleSession(),
          catalogRepository: catalog,
          overrides: FakeDependencyOverrides(watchHistory: history),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField), 'clip');
    await tester.tap(find.widgetWithText(FilledButton, 'Search'));
    await tester.pumpAndSettle();
    expect(find.text('Already watched clip'), findsOneWidget);

    await tester.tap(find.text('Home'));
    await tester.pumpAndSettle();
    await history.record(
      WatchHistoryEntry.fromPost(stale, DateTime.utc(2026, 8, 15)),
    );
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('Already watched clip'), findsNothing);
    expect(find.text('Fresh clip'), findsOneWidget);
  });
}
