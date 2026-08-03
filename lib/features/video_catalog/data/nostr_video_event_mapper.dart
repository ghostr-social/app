import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/data/creator_profile_summary.dart';
import 'package:ghostr/features/video_catalog/data/nostr_video_media.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_hashtags.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ndk/ndk.dart';

class NostrVideoEventMapper {
  const NostrVideoEventMapper();

  VideoPost map(Nip01Event event, Metadata? metadata) {
    try {
      return _map(event, metadata);
    } on AppFailure {
      rethrow;
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'NostrVideoEventMapper.map',
        message: 'Nostr video event is malformed.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }

  VideoPost _map(Nip01Event event, Metadata? metadata) {
    final media = _requiredVideoMedia(event);
    return VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse(event.id),
        creator: creatorProfileSummary(event.pubKey, metadata),
        nostrReference: _reference(event),
      ),
      content: VideoPostContent(
        caption: captionWithoutMediaUrls(event.content, media.urls),
        songName: _firstTag(event.tags, 'title') ?? 'Original sound',
        media: _source(media, event.id),
        publishedAt: _publishedAt(event.createdAt),
        hashtags: _hashtags(event),
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }

  VideoMediaSource _source(NostrVideoMedia media, String eventId) {
    var source = VideoMediaSource.remote(
      media.urls.first,
      fallbackUrls: media.urls.skip(1).toList(),
      delivery: media.delivery,
    );
    final digest = media.expectedSha256;
    if (digest != null) {
      source = VideoMediaSource.withExpectedSha256(source, digest);
    }
    return VideoMediaSource.withCacheScope(source, eventId);
  }

  NostrVideoMedia _requiredVideoMedia(Nip01Event event) {
    final media = NostrVideoMedia.fromEvent(
      tags: event.tags,
      content: event.content,
    );
    if (media == null) {
      throw const AppFailure('Nostr video event has no playable media.');
    }
    return media;
  }

  DateTime _publishedAt(int createdAt) =>
      DateTime.fromMillisecondsSinceEpoch(createdAt * 1000, isUtc: true);

  NostrEventReference _reference(Nip01Event event) {
    return NostrEventReference(
      eventId: NostrEventId.parse(event.id),
      authorPublicKeyHex: NostrPublicKeyHex.parse(event.pubKey),
      kind: NostrEventKind.parse(event.kind),
      identifier: _identifier(event),
    );
  }

  NostrEventIdentifier? _identifier(Nip01Event event) {
    if (event.kind < 30000 || event.kind >= 40000) return null;
    final value = _firstTag(event.tags, 'd');
    if (value == null || value.trim().isEmpty) {
      throw const AppFailure('Addressable Nostr video has no identifier.');
    }
    return NostrEventIdentifier.parse(value);
  }

  List<String> _hashtags(Nip01Event event) {
    final found = <String>{};
    for (final tag in event.tags.where((tag) => tag.firstOrNull == 't')) {
      final value = tag.elementAtOrNull(1);
      final normalized = value == null ? null : normalizeHashtag(value);
      if (normalized != null) found.add(normalized);
    }
    found.addAll(extractHashtags(event.content));
    return List<String>.unmodifiable(found);
  }

  String? _firstTag(List<List<String>> tags, String name) {
    return tags
        .where((tag) => tag.firstOrNull == name)
        .firstOrNull
        ?.elementAtOrNull(1);
  }
}
