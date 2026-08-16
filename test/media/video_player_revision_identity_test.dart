import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_video_id.dart';
import 'package:ghostr/platform/media/video_player_playback_port.dart';
import 'package:ghostr/shared/media/video_playback_port.dart';

import '../support/sample_data.dart';

void main() {
  test('different logical videos keep distinct playback identities', () {
    final port = VideoPlayerPlaybackPort();
    final media = samplePost().media;

    final initial = port.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: media,
        videoId: PlaybackVideoId.parse('initial-event'),
        isActive: true,
      ),
    );
    final revised = port.buildSurface(
      VideoPlaybackSurfaceRequest(
        media: media,
        videoId: PlaybackVideoId.parse('revised-event'),
        isActive: true,
      ),
    );

    expect(revised.key, isNot(initial.key));
  });
}
