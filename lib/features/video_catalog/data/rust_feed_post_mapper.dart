import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ndk/ndk.dart';

/// Maps one Rust feed row onto the domain post. The ndk mapper
/// (nostr_video_event_mapper.dart) is the parity spec: the post id
/// stays the Nostr event id and creators arrive with the ndk display
/// fallbacks already applied by the Rust profile store.
class RustFeedPostMapper {
  const RustFeedPostMapper();

  VideoPost map(FfiFeedPost post) {
    try {
      return _map(post);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'RustFeedPostMapper.map',
        message: 'Rust feed post is malformed.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  VideoPost _map(FfiFeedPost post) {
    return VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse(post.eventId),
        creator: _creator(post.creator),
        // Kind and identifier do not cross the feed FFI yet; social
        // interactions rejoin the Rust rows in a later phase-2 step.
        nostrReference: null,
      ),
      content: VideoPostContent(
        caption: post.caption,
        // The FFI row carries no `title` tag; the ndk mapper's
        // fallback applies unconditionally.
        songName: 'Original sound',
        media: _media(post),
        publishedAt: _publishedAt(post.createdAt),
        hashtags: List<String>.unmodifiable(post.hashtags),
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }

  /// Display fields come from the Rust profile store, which mirrors
  /// creator_profile_summary.dart; the id must still be the npub the
  /// profile routes expect.
  ProfileSummary _creator(FfiFeedCreator creator) {
    return ProfileSummary(
      id: ProfileId.parse(Nip19.encodePubKey(creator.pubkey)),
      displayName: creator.displayName,
      handle: creator.handle,
      avatarUrl: creator.avatarUrl,
    );
  }

  /// The engine's gateway post id becomes the cache scope so focus
  /// updates and playback URLs address the same Rust store entry
  /// (ffi_focus_item_media_mapper.dart prefers the scope).
  VideoMediaSource _media(FfiFeedPost post) {
    final media = post.media;
    if (media.urls.isEmpty) {
      throw const AppFailure('Nostr video event has no playable media.');
    }
    var source = VideoMediaSource.remote(
      media.urls.first,
      fallbackUrls: media.urls.skip(1).toList(),
      delivery: _delivery(media.delivery),
      metadata: VideoMediaMetadata(
        sizeBytes: media.sizeBytes?.toInt(),
        durationMs: media.durationMs?.toInt(),
      ),
    );
    final digest = media.sha256;
    if (digest != null) {
      source = VideoMediaSource.withExpectedSha256(source, digest);
    }
    return VideoMediaSource.withCacheScope(source, post.postId);
  }

  /// Round-trips `FfiFeedMedia.delivery` (api::feed_types contract).
  VideoMediaDelivery _delivery(String name) {
    return switch (name) {
      'progressive' => VideoMediaDelivery.progressive,
      'hls' => VideoMediaDelivery.hls,
      _ => throw AppFailure('Rust feed delivery kind is unknown: $name.'),
    };
  }

  DateTime _publishedAt(BigInt createdAt) {
    return DateTime.fromMillisecondsSinceEpoch(
      createdAt.toInt() * 1000,
      isUtc: true,
    );
  }
}
