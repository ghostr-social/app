import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/nostr/signed_nostr_event_json.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_repost_attribution.dart';
import 'package:ghostr/src/rust/api/delivery_types.dart';
import 'package:ghostr/src/rust/api/feed_types.dart';
import 'package:ndk/ndk.dart';

/// Maps one Rust feed row onto the domain post. The post id stays the
/// Nostr event id and the Rust profile store applies creator fallbacks.
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
        nostrReference: _reference(post),
        repost: _repost(post.repost),
      ),
      content: VideoPostContent(
        caption: post.caption,
        songName: post.title ?? 'Original sound',
        media: _media(post),
        publishedAt: _publishedAt(post.createdAt),
        hashtags: List<String>.unmodifiable(post.hashtags),
      ),
      // Interaction repositories hydrate these counts from Rust queries.
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }

  /// The event a like, comment, or report addresses.
  NostrEventReference _reference(FfiFeedPost post) {
    return NostrEventReference(
      eventId: NostrEventId.parse(post.eventId),
      authorPublicKeyHex: NostrPublicKeyHex.parse(post.creator.pubkey),
      kind: NostrEventKind.parse(post.eventKind),
      details: NostrEventReferenceDetails(
        identifier: _identifier(post),
        publishedIdentifier: _publishedIdentifier(post),
        signedEvent: _signedEvent(post.signedEventJson),
        isProtected: post.isProtected,
      ),
    );
  }

  SignedNostrEventJson? _signedEvent(String? raw) {
    return raw == null ? null : SignedNostrEventJson.parse(raw);
  }

  VideoRepostAttribution? _repost(FfiFeedRepost? repost) {
    if (repost == null) return null;
    return VideoRepostAttribution(
      eventId: NostrEventId.parse(repost.eventId),
      reposter: _creator(repost.reposter),
      repostedAt: _publishedAt(repost.repostedAt),
      target: switch (repost.target) {
        FfiFeedRepostTarget.specificEvent => VideoRepostTarget.specificEvent,
        FfiFeedRepostTarget.coordinate => VideoRepostTarget.coordinate,
      },
    );
  }

  /// Addressable rows must name their `d` tag or the coordinate a
  /// social write targets would silently degrade to the event id.
  NostrEventIdentifier? _identifier(FfiFeedPost post) {
    if (post.eventKind < 30000 || post.eventKind >= 40000) return null;
    final value = post.identifier;
    if (value == null) {
      throw const AppFailure('Addressable Nostr video has no identifier.');
    }
    return NostrEventIdentifier.parse(value);
  }

  NostrEventIdentifier? _publishedIdentifier(FfiFeedPost post) {
    final value = post.publishedIdentifier;
    return value == null ? null : NostrEventIdentifier.published(value);
  }

  /// Display fields come from the Rust profile store; the id remains the
  /// npub identity that profile routes expect.
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
        blurhash: media.blurhash,
      ),
    );
    final digest = media.sha256;
    if (digest != null) {
      source = VideoMediaSource.withExpectedSha256(source, digest);
    }
    return VideoMediaSource.withCacheScope(source, post.postId);
  }

  /// Round-trips `FfiFeedMedia.delivery` (api::feed_types contract).
  VideoMediaDelivery _delivery(FfiMediaDelivery delivery) {
    return switch (delivery) {
      FfiMediaDelivery.progressive => VideoMediaDelivery.progressive,
      FfiMediaDelivery.hls => VideoMediaDelivery.hls,
    };
  }

  DateTime _publishedAt(BigInt createdAt) {
    return DateTime.fromMillisecondsSinceEpoch(
      createdAt.toInt() * 1000,
      isUtc: true,
    );
  }
}
