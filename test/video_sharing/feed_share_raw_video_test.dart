import 'dart:async';

import 'package:flutter_test/flutter_test.dart';

import '../support/fake_video_catalog_repository.dart';
import '../support/fake_video_sharing.dart';
import '../support/feed_screen_harness.dart';
import '../support/sample_data.dart';

void main() {
  testWidgets('downloads the video before opening file sharing', (
    tester,
  ) async {
    final pending = Completer<void>();
    final sharing = FakeVideoShareWorkflow(pending: pending);
    final post = samplePost();
    final repository = FakeVideoCatalogRepository(forYouFeed: [post]);
    await tester.pumpWidget(
      feedScreenHarness(repository, shareWorkflow: sharing),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Share video'));
    await tester.pump();

    expect(sharing.requests.single.media, same(post.media));
    expect(sharing.requests.single.origin.width, greaterThan(0));
    expect(find.bySemanticsLabel('Downloading video to share'), findsOneWidget);
    pending.complete();
    await tester.pumpAndSettle();

    expect(find.byTooltip('Share video'), findsOneWidget);
  });
}
