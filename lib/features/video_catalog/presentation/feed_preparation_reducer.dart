import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/core/media/prepared_progressive_playback.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

part 'feed_preparation_matching.dart';

final class FeedPlaybackPreparation {
  factory FeedPlaybackPreparation.unmanaged({
    BigInt? revision,
    List<PreparedProgressivePlayback> upcoming = const [],
  }) => FeedPlaybackPreparation._(
    revision,
    null,
    List.unmodifiable(upcoming),
    false,
  );

  factory FeedPlaybackPreparation.managed({
    required BigInt revision,
    PreparedProgressivePlayback? current,
    List<PreparedProgressivePlayback> upcoming = const [],
  }) => FeedPlaybackPreparation._(
    revision,
    current,
    List.unmodifiable(upcoming),
    true,
  );

  const FeedPlaybackPreparation._(
    this.revision,
    this.current,
    this.upcoming,
    this._managesCurrent,
  );

  final BigInt? revision;
  final PreparedProgressivePlayback? current;
  final List<PreparedProgressivePlayback> upcoming;
  final bool _managesCurrent;

  bool get isManaged => _managesCurrent;

  PreparedProgressivePlayback? get next {
    return upcoming.isEmpty ? null : upcoming.first;
  }

  PreparedProgressivePlayback? forMedia(VideoMediaSource media) {
    if (current?.matches(media) == true) return current;
    return upcoming.where((asset) => asset.matches(media)).firstOrNull;
  }
}

final class FeedPreparationReducer {
  BigInt _watermark = BigInt.zero;
  PlaybackPreparationPlan? _latest;

  BigInt get watermark => _watermark;

  bool observe(PlaybackPreparationPlan plan) {
    if (plan.revision < _watermark) return false;
    _watermark = plan.revision;
    _latest = plan;
    return true;
  }

  FeedPlaybackPreparation? accept(
    PlaybackPreparationPlan plan,
    VideoMediaSource current,
    VideoMediaSource? next,
  ) => acceptWindow(plan, current, [if (next != null) next]);

  FeedPlaybackPreparation? acceptWindow(
    PlaybackPreparationPlan plan,
    VideoMediaSource current,
    List<VideoMediaSource> upcoming,
  ) {
    if (!observe(plan)) return null;
    return _project(plan, current, upcoming);
  }

  FeedPlaybackPreparation waiting() {
    return FeedPlaybackPreparation.managed(revision: _watermark);
  }

  FeedPlaybackPreparation project(
    VideoMediaSource current,
    VideoMediaSource? next,
  ) => projectWindow(current, [if (next != null) next]);

  FeedPlaybackPreparation projectWindow(
    VideoMediaSource current,
    List<VideoMediaSource> upcoming,
  ) {
    final latest = _latest;
    if (!current.canCacheAsSingleFile) {
      return latest == null
          ? FeedPlaybackPreparation.unmanaged()
          : _unmanagedWindow(latest, upcoming);
    }
    return latest == null ? waiting() : _project(latest, current, upcoming);
  }

  FeedPlaybackPreparation realign(
    FeedPlaybackPreparation previous,
    VideoMediaSource current,
    VideoMediaSource? next,
  ) => realignWindow(previous, current, [if (next != null) next]);

  FeedPlaybackPreparation realignWindow(
    FeedPlaybackPreparation previous,
    VideoMediaSource current,
    List<VideoMediaSource> upcoming,
  ) {
    final promoted = _matchingPrepared(previous.upcoming, current);
    if (promoted != null && promoted.matches(current)) {
      return FeedPlaybackPreparation.managed(
        revision: previous.revision!,
        current: promoted,
        upcoming: _retained(previous.upcoming, upcoming),
      );
    }
    return projectWindow(current, upcoming);
  }

  FeedPlaybackPreparation _project(
    PlaybackPreparationPlan plan,
    VideoMediaSource currentMedia,
    List<VideoMediaSource> upcomingMedia,
  ) {
    if (!currentMedia.canCacheAsSingleFile) {
      return _unmanagedWindow(plan, upcomingMedia);
    }
    final currentId = currentMedia.playbackDeliveryId;
    if (plan.currentDeliveryId == currentId) {
      return _matchingWindow(plan, currentMedia, upcomingMedia);
    }
    final promoted = _matchingAsset(plan.upcoming, currentMedia);
    return FeedPlaybackPreparation.managed(
      revision: plan.revision,
      current: promoted,
      upcoming: _startableWindow(plan.upcoming, upcomingMedia),
    );
  }

  FeedPlaybackPreparation _unmanagedWindow(
    PlaybackPreparationPlan plan,
    List<VideoMediaSource> upcomingMedia,
  ) {
    return FeedPlaybackPreparation.unmanaged(
      revision: plan.revision,
      upcoming: _startableWindow([
        if (plan.current != null) plan.current!,
        ...plan.upcoming,
      ], upcomingMedia),
    );
  }

  FeedPlaybackPreparation _matchingWindow(
    PlaybackPreparationPlan plan,
    VideoMediaSource current,
    List<VideoMediaSource> upcoming,
  ) {
    return FeedPlaybackPreparation.managed(
      revision: plan.revision,
      current: _matching(plan.current, current),
      upcoming: _startableWindow(plan.upcoming, upcoming),
    );
  }
}
