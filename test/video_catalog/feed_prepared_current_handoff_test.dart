import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/gateway_video_playback_port.dart';
import 'package:ghostr/platform/media/hls_video_playback_port.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';

import '../support/fake_hls_playback_gateway.dart';
import '../support/fake_progressive_playback_gateway.dart';
import '../support/feed_preparation_fixture.dart';

void main() {
  testWidgets('exact preparation keeps the current native player', (
    tester,
  ) async {
    final fixture = FeedPreparationFixture();
    final currentUrl = fixture.url('p0');
    final gateway = FakeProgressivePlaybackGateway(
      immediatePlaybackUrl: currentUrl,
    );
    final hls = FakeHlsPlaybackGateway();
    final playback = HlsVideoPlaybackPort(
      gateway: hls,
      delegate: GatewayVideoPlaybackPort(
        delegate: VideoPlayerPlaybackPort(),
        gateway: gateway,
      ),
    );
    await fixture.pump(tester, playbackPort: playback);

    expect(fixture.platform.creationsFor(currentUrl), 1);
    fixture.publish(1, 'p0', null);
    await _turn(tester);

    expect(gateway.requests, hasLength(1));
    expect(hls.requests, isEmpty);
    expect(fixture.platform.creationsFor(currentUrl), 1);
    expect(fixture.platform.playerCount, 1);
  });
}

Future<void> _turn(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await tester.pump();
}
