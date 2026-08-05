import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';

import '../support/live_video_search_updates.dart';
import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('a late Rust match replaces the searching-empty panel',
      (tester) async {
    final updates = LiveVideoSearchUpdates();
    addTearDown(updates.close);
    await tester.pumpWidget(searchScreenHarness(
      PagedSearchRepository(pages: const [<Never>[]]),
      updates: updates,
    ));

    await tester.enterText(find.byType(TextField), 'ghost');
    await tester.tap(find.text('Search'));
    await tester.pump();
    expect(find.text('No matches yet'), findsOneWidget);

    updates.publish(
      'ghost',
      VideoFeedPage(
        posts: [samplePost(id: 'late', caption: 'Found over time')],
      ),
    );
    await tester.pump();
    expect(find.text('Found over time'), findsOneWidget);
    expect(find.text('No matches yet'), findsNothing);
  });
}
