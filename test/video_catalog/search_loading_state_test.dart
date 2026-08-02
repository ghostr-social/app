import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

import '../support/search_screen_harness.dart';

void main() {
  testWidgets('announces the Nostr search loading state', (tester) async {
    await tester.pumpWidget(searchScreenHarness(_PendingSearchRepository()));
    await tester.enterText(find.byType(TextField), 'nostr');
    await tester.tap(find.text('Search'));
    await tester.pump();

    expect(find.bySemanticsLabel('Searching Nostr'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}

class _PendingSearchRepository implements VideoSearchRepository {
  final _search = Completer<List<VideoPost>>();

  @override
  Future<List<VideoPost>> search(String query) => _search.future;
}
