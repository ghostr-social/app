import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('a superseded gateway answer cannot replace newer media', () async {
    final gateway = FakeProgressivePlaybackGateway();
    final first = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/first.mp4'),
      'post-1',
    );
    final second = VideoMediaSource.withCacheScope(
      VideoMediaSource.remote('https://media.test/second.mp4'),
      'post-2',
    );
    final cubit = GatewayPlaybackCubit(gateway);

    final firstLoad = cubit.load(first);
    final secondLoad = cubit.load(second);
    gateway.completeNext(playbackUrl: fakeProgressivePlaybackUrl);
    await firstLoad;

    expect(cubit.state, isA<GatewayPlaybackPreparing>());

    const secondUrl =
        'http://127.0.0.1:3210/video.mp4?id=post-2&cap='
        'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
    gateway.completeNext(playbackUrl: secondUrl);
    await secondLoad;

    expect((cubit.state as GatewayPlaybackReady).media.remoteUrl, secondUrl);
    await cubit.close();
  });
}
