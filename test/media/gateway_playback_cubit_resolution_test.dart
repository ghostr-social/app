import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('resolves progressive media into an explicit ready state', () async {
    final gateway = FakeProgressivePlaybackGateway();
    final media = VideoMediaSource.remote('https://media.test/clip.mp4');
    final cubit = GatewayPlaybackCubit(gateway);

    expect(cubit.state, isA<GatewayPlaybackPreparing>());

    final resolving = cubit.load(media);
    expect(gateway.requests, [media]);
    gateway.completeNext();
    await resolving;

    final ready = cubit.state as GatewayPlaybackReady;
    expect(ready.media.remoteUrl, fakeProgressivePlaybackUrl);

    await cubit.close();
  });
}
