import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../support/fakes.dart';
import '../support/recording_video_playback_port.dart';
import '../support/sample_data.dart';
import '../support/test_app.dart';

void main() {
  testWidgets('covering the feed with a profile deactivates playback',
      (tester) async {
    final post = samplePost();
    final playback = RecordingVideoPlaybackPort();
    final dependencies = buildFakeDependencies(
      session: sampleSession(),
      catalogRepository: FakeVideoCatalogRepository(
        forYouFeed: [post],
        feed: FakeFeedScenario(
          profiles: {post.creator.id: sampleProfileDetails()},
        ),
      ),
      device: FakeDeviceDependencies(playback: playback),
    );
    await tester.pumpWidget(buildTestApp(dependencies));
    await tester.pumpAndSettle();
    expect(playback.activity[post.media.debugLabel]!.last, isTrue);

    await tester.tap(find.widgetWithText(FilledButton, 'Profile'));
    await tester.pumpAndSettle();

    expect(playback.activity[post.media.debugLabel]!.last, isFalse);

    await tester.pageBack();
    await tester.pumpAndSettle();

    expect(playback.activity[post.media.debugLabel]!.last, isTrue);
  });
}
