import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('a failed empty live search shows a retryable error',
      (tester) async {
    final updates = LiveVideoSearchUpdates();
    addTearDown(updates.close);
    await tester.pumpWidget(searchScreenHarness(
      PagedSearchRepository(pages: const [<Never>[]]),
      updates: updates,
    ));

    await tester.enterText(find.byType(TextField), 'ghost');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    expect(find.text('No matches yet'), findsOneWidget);

    updates.fail('ghost', StateError('native watcher stopped'));
    await tester.pumpAndSettle();

    expect(find.text('Search unavailable'), findsOneWidget);
    expect(find.text('Live search updates paused.'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
    expect(find.text('No matches yet'), findsNothing);
  });
}
