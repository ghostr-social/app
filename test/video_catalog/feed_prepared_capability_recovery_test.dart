import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_inventory/domain/playback_recovery_policy.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/fake_progressive_playback_gateway.dart';
import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('failed prepared capability renews from its exact origin', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    final initial = fixture.url('p0');
    final gateway = FakeProgressivePlaybackGateway(
      immediatePlaybackUrl: initial,
    );
    final playback = GatewayVideoPlaybackPort(
      delegate: VideoPlayerPlaybackPort(
        recoveryPolicy: PlaybackRecoveryPolicy([Duration.zero]),
      ),
      gateway: gateway,
    );
    await fixture.pump(tester, playbackPort: playback);
    fixture.publish(1, 'p0', null);
    await _turn(tester);

    gateway.resolveImmediatelyWith(_renewedUrl);
    fixture.platform.fail(fixture.platform.playerFor(initial));
    await _turn(tester);
    await _turn(tester);

    expect(gateway.requests, hasLength(2));
    expect(gateway.requests[1], same(fixture.posts.first.media));
    expect(fixture.platform.creationsFor(_renewedUrl), 1);
    expect(find.text('Video unavailable'), findsNothing);
  });
}

Future<void> _turn(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump();
}

const _renewedUrl =
    'http://127.0.0.1:4040/video.mp4?id=p0&cap='
    'ddddddddddddddddddddddddddddddddddddddddddd';
