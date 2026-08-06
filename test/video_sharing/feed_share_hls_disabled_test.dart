import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_sharing.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('does not offer a playlist URL as a shareable video', (
    tester,
  ) async {
    final sharing = FakeVideoShareWorkflow(isSupported: false);
    final post = samplePost().withMedia(
      VideoMediaSource.remote(
        'https://media.test/playlist.m3u8',
        delivery: VideoMediaDelivery.hls,
      ),
    );
    final repository = FakeVideoCatalogRepository(forYouFeed: [post]);
    await tester.pumpWidget(
      feedScreenHarness(repository, shareWorkflow: sharing),
    );
    await tester.pumpAndSettle();

    final button = tester.widget<IconButton>(
      find.byWidgetPredicate(
        (widget) =>
            widget is IconButton &&
            widget.tooltip == 'Sharing unavailable for this video',
      ),
    );

    expect(button.onPressed, isNull);
    expect(sharing.requests, isEmpty);
  });
}
