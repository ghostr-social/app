import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/paged_search_repository.dart';
import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('reports an older-page failure without hiding results',
      (tester) async {
    final repository = PagedSearchRepository(pages: [
      [samplePost(caption: 'Visible clip')],
      [samplePost(id: 'older')],
    ]);
    await tester.pumpWidget(searchScreenHarness(repository));
    await tester.enterText(find.byType(TextField), 'ghost');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    repository.videosFailure = StateError('relay unavailable');

    await tester.drag(find.byType(CustomScrollView), const Offset(0, -300));
    await tester.pumpAndSettle();

    expect(find.text('Visible clip'), findsOneWidget);
    expect(
      find.text('Older search results are unavailable right now.'),
      findsOneWidget,
    );
  });
}
