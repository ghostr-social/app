import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import '../support/ready_playback_preparation.dart';
import '../support/sample_data.dart';

void main() {
  test('promotion never advertises an unready outgoing asset for reversal', () {
    final previousMedia = samplePost(id: 'previous').media;
    final nextMedia = samplePost(id: 'next').media;
    final previousAsset = readyPlaybackPreparation(previousMedia);
    final nextAsset = readyPlaybackPreparation(nextMedia);
    final preparing = PlaybackPreparationAsset(
      authority: previousAsset.authority,
      media: previousAsset.media,
      readiness: PlaybackPreparationReadiness.preparing,
    );
    final previous = FeedPlaybackPreparation.managed(
      revision: BigInt.one,
      current: preparing.bind(previousMedia),
      upcoming: [nextAsset.bind(nextMedia)],
    );

    final promoted = FeedPreparationReducer().realignWindow(
      previous,
      nextMedia,
      [previousMedia],
    );

    expect(promoted.current?.authority, nextAsset.authority);
    expect(promoted.forMedia(previousMedia), isNull);
  });
}
