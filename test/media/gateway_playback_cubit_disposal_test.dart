import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/platform/media/gateway_playback_cubit.dart';

import '../support/fake_progressive_playback_gateway.dart';

void main() {
  test('a gateway answer cannot emit after playback is disposed', () async {
    final gateway = FakeProgressivePlaybackGateway();
    final media = VideoMediaSource.remote('https://media.test/clip.mp4');
    final cubit = GatewayPlaybackCubit(gateway, media);

    final loading = cubit.load(media);
    await cubit.close();
    gateway.completeNext();
    await loading;

    expect(cubit.state, isA<GatewayPlaybackPreparing>());
  });
}
