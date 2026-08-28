import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('direct playback admits two warm previous players', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture(postCount: 4);
    addTearDown(fixture.updates.close);
    for (var index = 0; index < fixture.posts.length; index += 1) {
      fixture.posts[index] = fixture.posts[index].withMedia(
        VideoMediaSource.local('/tmp/p$index.mp4'),
      );
    }
    await fixture.pump(tester, playbackPort: VideoPlayerPlaybackPort());
    final first = fixture.platform.playerFor(_url(fixture, 0));

    await _swipe(tester);
    await fixture.settle(tester);
    final previous = fixture.platform.playerFor(_url(fixture, 1));
    await _swipe(tester);
    await fixture.settle(tester);
    final current = fixture.platform.playerFor(_url(fixture, 2));

    expect(fixture.platform.disposed, isNot(contains(first)));
    expect(fixture.platform.disposed, isNot(contains(previous)));
    expect(fixture.platform.isPlaying(current), isTrue);
    expect(fixture.platform.playerCount, 3);
    expect(fixture.platform.audibleOverlap, isFalse);
  });
}

String _url(FeedPreparationFixture fixture, int index) {
  return fixture.posts[index].media.localPath!;
}

Future<void> _swipe(WidgetTester tester) async {
  final page = find.byType(PageView);
  final gesture = await tester.startGesture(tester.getCenter(page));
  await gesture.moveBy(Offset(0, -tester.getSize(page).height * 0.23));
  await tester.pump(const Duration(milliseconds: 16));
  await gesture.up();
  await tester.pump(const Duration(milliseconds: 500));
}
