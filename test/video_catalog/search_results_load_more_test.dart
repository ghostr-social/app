import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_feed_page.dart';
import 'package:ghostr/features/video_catalog/domain/video_search_repository.dart';

import '../support/sample_data.dart';
import '../support/search_screen_harness.dart';

void main() {
  testWidgets('scrolling the results loads and appends the next page',
      (tester) async {
    final repository = _TwoPageRepository();
    await tester.pumpWidget(searchScreenHarness(repository));
    await tester.enterText(find.byType(TextField), 'ghost');
    await tester.tap(find.text('Search'));
    await tester.pumpAndSettle();
    expect(find.text('First clip'), findsOneWidget);

    await tester.drag(find.byType(CustomScrollView), const Offset(0, -300));
    await tester.pump();
    expect(repository.olderThans.whereType<DateTime>(), isNotEmpty);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    repository.second.complete(VideoFeedPage(
      posts: [samplePost(id: 'older', caption: 'Older clip')],
    ));
    await tester.pumpAndSettle();

    expect(find.text('Older clip'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });
}

class _TwoPageRepository implements VideoSearchRepository {
  final second = Completer<VideoFeedPage>();
  final olderThans = <DateTime?>[];

  @override
  Future<VideoFeedPage> searchVideos(String query, {DateTime? olderThan}) {
    olderThans.add(olderThan);
    if (olderThan != null) return second.future;
    return Future.value(VideoFeedPage(
      posts: [samplePost(id: 'first', caption: 'First clip')],
      nextOlderThan: DateTime.utc(2026, 1, 1),
    ));
  }

  @override
  Future<List<ProfileSummary>> searchCreators(String query) async {
    return const <ProfileSummary>[];
  }
}
