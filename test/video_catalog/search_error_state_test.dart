import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('shows a retryable search error', (tester) async {
    final repository = _FailingSearchRepository();
    await tester.pumpWidget(searchScreenHarness(repository));

    await tester.enterText(find.byType(TextField), 'relay');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();

    expect(find.text('Search unavailable'), findsOneWidget);
    expect(find.text('Search failed.'), findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });
}

class _FailingSearchRepository extends FakeVideoCatalogRepository {
  _FailingSearchRepository() : super(forYouFeed: []);

  @override
  Future<List<VideoPost>> search(String query) {
    throw const AppFailure('Search failed.');
  }
}
