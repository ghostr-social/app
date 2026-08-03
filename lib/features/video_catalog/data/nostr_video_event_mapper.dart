import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/errors/boundary_failure.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/media/video_sha256.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
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
    final npub = Nip19.encodePubKey(event.pubKey);
    final media = _requiredVideoMedia(event.tags);
    return VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse(event.id),
        creator: _profile(npub, event.pubKey, metadata),
        nostrReference: _reference(event),
      ),
      content: VideoPostContent(
        caption: event.content,
        songName: _firstTag(event.tags, 'title') ?? 'Original sound',
        media: _source(media, event.id),
        publishedAt: _publishedAt(event.createdAt),
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }

  VideoMediaSource _source(_NostrVideoMedia media, String eventId) {
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

  _NostrVideoMedia _requiredVideoMedia(List<List<String>> tags) {
    final media = _videoMedia(tags);
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

  ProfileSummary _profile(
    String npub,
    String publicKey,
    Metadata? metadata,
  ) {
    final name = metadata?.getName();
    final hasName = name != null && name != publicKey && name.trim().isNotEmpty;
    return ProfileSummary(
      id: ProfileId.parse(npub),
      displayName: hasName ? name : '${npub.substring(0, 12)}…',
      handle: '@$npub',
      avatarUrl: metadata?.picture,
    );
  }

  _NostrVideoMedia? _videoMedia(List<List<String>> tags) {
    for (final tag in tags.where((tag) => tag.firstOrNull == 'imeta')) {
      final media = _tryVideoMedia(tag);
      if (media != null) return media;
    }
    return null;
  }

  _NostrVideoMedia? _tryVideoMedia(List<String> tag) {
    final mimeType = _imetaField(tag, 'm');
    if (!_isVideoMime(mimeType)) return null;
    final digest = _videoDigest(tag);
    if (!digest.valid) return null;
    final urls = _videoUrls(tag);
    if (urls.isEmpty) return null;
    return _NostrVideoMedia(urls, _delivery(mimeType!), digest.value?.value);
  }

  ({bool valid, VideoSha256? value}) _videoDigest(List<String> tag) {
    final rawDigest = _imetaField(tag, 'x');
    final value = rawDigest == null ? null : VideoSha256.tryParse(rawDigest);
    return (valid: rawDigest == null || value != null, value: value);
  }

  List<String> _videoUrls(List<String> tag) {
    final primary = _imetaField(tag, 'url');
    return <String>{
      if (_isHttpUrl(primary)) primary!,
      ..._validFallbacks(tag),
    }.toList();
  }

  List<String> _validFallbacks(List<String> tag) {
    return tag
        .skip(1)
        .where((field) => field.startsWith('fallback '))
        .map((field) => field.substring('fallback '.length))
        .where(_isHttpUrl)
        .toList();
  }

  String? _imetaField(List<String> tag, String name) {
    for (final field in tag.skip(1)) {
      if (field.startsWith('$name ')) return field.substring(name.length + 1);
    }
    return null;
  }

  String? _firstTag(List<List<String>> tags, String name) {
    return tags
        .where((tag) => tag.firstOrNull == name)
        .firstOrNull
        ?.elementAtOrNull(1);
  }

  bool _isHttpUrl(String? value) {
    final uri = value == null ? null : Uri.tryParse(value);
    return uri != null &&
        (uri.scheme == 'https' || uri.scheme == 'http') &&
        uri.host.isNotEmpty;
  }

  bool _isVideoMime(String? value) {
    return _normalizedMime(value)?.startsWith('video/') == true ||
        _isHlsMime(value);
  }

  VideoMediaDelivery _delivery(String mimeType) {
    return _isHlsMime(mimeType)
        ? VideoMediaDelivery.hls
        : VideoMediaDelivery.progressive;
  }

  bool _isHlsMime(String? value) {
    return const {
      'application/x-mpegurl',
      'application/vnd.apple.mpegurl',
    }.contains(_normalizedMime(value));
  }

  String? _normalizedMime(String? value) => value?.trim().toLowerCase();
}

class _NostrVideoMedia {
  const _NostrVideoMedia(this.urls, this.delivery, this.expectedSha256);

  final List<String> urls;
  final VideoMediaDelivery delivery;
  final String? expectedSha256;
}
