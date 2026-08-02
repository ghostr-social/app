import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/feed_kind.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';

import '../support/fakes.dart';
import '../support/feed_screen_harness.dart';

void main() {
  testWidgets('announces the video-feed loading state', (tester) async {
    await tester.pumpWidget(feedScreenHarness(_PendingFeedRepository()));
    await tester.pump();

    expect(find.bySemanticsLabel('Loading video feed'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
  });
}

class _PendingFeedRepository extends FakeVideoCatalogRepository {
  _PendingFeedRepository() : super(forYouFeed: []);

  final _load = Completer<List<VideoPost>>();

  @override
  Future<List<VideoPost>> loadFeed(FeedKind kind) => _load.future;
}
