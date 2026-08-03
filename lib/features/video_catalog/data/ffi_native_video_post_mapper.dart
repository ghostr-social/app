import 'dart:developer';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/data/ffi_video_event_matcher.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/src/rust/video/video.dart';

typedef FfiMediaSourceMapper = VideoMediaSource Function(FfiVideoDownload);

VideoPost? tryMapFfiNativeVideo(
  FfiVideoDownload video,
  FfiMediaSourceMapper mediaSource,
) {
  if (!ffiVideoCanMapWithoutSnapshot(video)) return null;
  try {
    return _mapNative(video, mediaSource);
  } on Object catch (error, stackTrace) {
    log(
      'Skipping malformed native video inventory.',
      name: 'ghostr.video.native',
      error: error,
      stackTrace: stackTrace,
    );
    return null;
  }
}

VideoPost _mapNative(
  FfiVideoDownload video,
  FfiMediaSourceMapper mediaSource,
) {
  final event = video.event;
  return VideoPost(
    identity: VideoPostIdentity(
      id: VideoPostId.parse(event.eventId),
      creator: _creator(video, event),
      nostrReference: _reference(event),
    ),
    content: _content(video, event, mediaSource),
    metrics: _metrics(video),
  );
}

NostrEventReference _reference(FfiNostrEventIdentity event) {
  return NostrEventReference(
    eventId: NostrEventId.parse(event.eventId),
    authorPublicKeyHex: NostrPublicKeyHex.parse(event.authorPublicKeyHex),
    kind: NostrEventKind.parse(event.kind.toInt()),
    identifier: _identifier(event.identifier),
  );
}

VideoPostMetrics _metrics(FfiVideoDownload video) {
  return VideoPostMetrics(
    likeCount: _count(video.nostr.likes),
    commentCount: _count(video.nostr.comments),
    viewerHasLiked: false,
  );
}

ProfileSummary _creator(
  FfiVideoDownload video,
  FfiNostrEventIdentity event,
) {
  final npub = video.nostr.user.npub?.trim();
  final id = npub == null || npub.isEmpty ? event.authorPublicKeyHex : npub;
  final name = video.nostr.user.name?.trim();
  return ProfileSummary(
    id: ProfileId.parse(id),
    displayName: name == null || name.isEmpty ? _short(id) : name,
    handle: '@${_short(id)}',
    avatarUrl: video.nostr.user.profilePicture,
  );
}

VideoPostContent _content(
  FfiVideoDownload video,
  FfiNostrEventIdentity event,
  FfiMediaSourceMapper mediaSource,
) {
  final content = event.content.trim();
  return VideoPostContent(
    caption: content.isEmpty ? video.nostr.title : content,
    songName: video.nostr.songName,
    media: mediaSource(video),
    publishedAt: DateTime.fromMillisecondsSinceEpoch(
      event.createdAt.toInt() * 1000,
      isUtc: true,
    ),
  );
}

NostrEventIdentifier? _identifier(String? raw) {
  final value = raw?.trim();
  return value == null || value.isEmpty
      ? null
      : NostrEventIdentifier.parse(value);
}

int _count(String raw) {
  final count = int.tryParse(raw);
  return count == null || count < 0 ? 0 : count;
}

String _short(String value) {
  return value.length <= 12 ? value : '${value.substring(0, 12)}…';
}
