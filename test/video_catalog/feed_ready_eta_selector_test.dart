import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_delivery_updates.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_ready_selector.dart';

import '../support/sample_data.dart';

void main() {
  test('waits briefly when the intended post has a short ETA', () {
    final fixture = _Fixture(eta: const Duration(milliseconds: 100));

    final decision = fixture.select();

    expect(decision.action, FeedReadyAction.wait);
    expect(decision.selectedIndex, 1);
  });

  test('rescues immediately when the intended ETA exceeds grace', () {
    final fixture = _Fixture(eta: const Duration(seconds: 1));

    final decision = fixture.select();

    expect(decision.action, FeedReadyAction.rescue);
    expect(decision.selectedIndex, 2);
  });

  test('rescues after the short-ETA grace expires', () {
    final fixture = _Fixture(eta: const Duration(milliseconds: 100));

    final decision = fixture.select(graceExpired: true);

    expect(decision.action, FeedReadyAction.rescue);
    expect(decision.reason, FeedReadyReason.graceExpired);
  });

  test('never substitutes outside the semantic top-K window', () {
    final fixture = _Fixture(eta: const Duration(seconds: 1), readyIndex: 4);

    final decision = fixture.select();

    expect(decision.action, FeedReadyAction.intended);
    expect(decision.selectedIndex, 1);
  });
}

final class _Fixture {
  _Fixture({required Duration eta, int readyIndex = 2})
    : posts = List.generate(5, (index) => samplePost(id: 'p$index')),
      _eta = eta,
      _readyIndex = readyIndex;

  final List<VideoPost> posts;
  final Duration _eta;
  final int _readyIndex;

  FeedReadyDecision select({bool graceExpired = false}) {
    final delivery = <PlaybackDeliveryId, VideoDeliverySnapshot>{
      posts[1].media.playbackDeliveryId!: _snapshot(1, false, _eta),
      posts[_readyIndex].media.playbackDeliveryId!: _snapshot(
        _readyIndex,
        true,
        Duration.zero,
      ),
    };
    return const FeedReadySelector().select(
      posts,
      fromIndex: 0,
      intendedIndex: 1,
      delivery: delivery,
      graceExpired: graceExpired,
    );
  }

  VideoDeliverySnapshot _snapshot(int index, bool ready, Duration eta) {
    return VideoDeliverySnapshot(
      deliveryId: posts[index].media.playbackDeliveryId!,
      phase: ready
          ? VideoDeliveryPhase.startable
          : VideoDeliveryPhase.preparing,
      bytesPresent: BigInt.zero,
      eta: eta,
    );
  }
}
