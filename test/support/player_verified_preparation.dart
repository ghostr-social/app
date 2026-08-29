import 'package:ghostr/core/media/playback_delivery_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/presentation/feed_preparation_reducer.dart';
import 'package:ghostr/features/video_inventory/domain/playback_preparation.dart';

import 'ready_playback_preparation.dart';

PlaybackPreparationPlan playerVerifiedPlan(
  List<VideoPost> posts, {
  required int currentIndex,
  required List<int> readyIndices,
  BigInt? revision,
}) {
  return PlaybackPreparationPlan(
    revision: revision ?? BigInt.one,
    currentDeliveryId: posts[currentIndex].media.playbackDeliveryId,
    upcoming: readyIndices
        .map((index) => readyPlaybackPreparation(posts[index].media))
        .toList(growable: false),
  );
}

FeedPlaybackPreparation playerVerifiedWindow(
  List<VideoPost> posts, {
  required int currentIndex,
  required List<int> readyIndices,
}) {
  final plan = playerVerifiedPlan(
    posts,
    currentIndex: currentIndex,
    readyIndices: readyIndices,
  );
  final upcoming = readyIndices.map((index) => posts[index].media).toList();
  return FeedPreparationReducer().acceptWindow(
    plan,
    posts[currentIndex].media,
    upcoming,
  )!;
}
