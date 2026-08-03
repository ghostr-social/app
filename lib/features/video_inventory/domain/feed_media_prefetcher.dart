import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_inventory/domain/video_inventory_port.dart';

/// Keeps the media cache warm around the viewer's position in the feed.
final class FeedMediaPrefetcher {
  const FeedMediaPrefetcher({
    required VideoInventoryPort inventory,
    int ahead = 6,
    int behind = 2,
  })  : _inventory = inventory,
        _ahead = ahead,
        _behind = behind;

  final VideoInventoryPort _inventory;
  final int _ahead;
  final int _behind;

  /// Queues background caching for the videos around [activeIndex],
  /// closest-ahead first.
  void focus(List<VideoPost> posts, int activeIndex) {
    final media = <VideoMediaSource>[
      for (var offset = 1; offset <= _ahead; offset += 1)
        if (activeIndex + offset < posts.length)
          posts[activeIndex + offset].media,
      for (var offset = 1; offset <= _behind; offset += 1)
        if (activeIndex - offset >= 0) posts[activeIndex - offset].media,
    ];
    if (media.isNotEmpty) _inventory.prepare(media);
  }
}
