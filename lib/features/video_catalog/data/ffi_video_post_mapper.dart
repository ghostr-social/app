import 'dart:developer';

import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';
import 'package:ghostr/src/rust/video/video.dart';

class FfiVideoPostMapper {
  const FfiVideoPostMapper(this.gatewayBaseUrl);

  final String gatewayBaseUrl;

  Iterable<VideoPost> map(
    List<FfiVideoDownload> nativeVideos,
    List<VideoPost> snapshot,
  ) sync* {
    final remaining = nativeVideos.toList();
    for (final post in snapshot.where(_isCanonical)) {
      final native = _takeMatching(post, remaining);
      if (native != null) yield post.withMedia(_mediaSource(native));
    }
    for (final native in remaining) {
      final post = _tryMapNative(native);
      if (post != null) yield post;
    }
  }

  bool _isCanonical(VideoPost post) => post.nostrReference != null;

  FfiVideoDownload? _takeMatching(
    VideoPost post,
    List<FfiVideoDownload> candidates,
  ) {
    final index = candidates.indexWhere((video) {
      final urls = {video.url, video.nostr.url};
      return post.media.remoteUrls.any(urls.contains);
    });
    return index < 0 ? null : candidates.removeAt(index);
  }

  VideoPost? _tryMapNative(FfiVideoDownload video) {
    try {
      return _mapNative(video);
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

  VideoPost _mapNative(FfiVideoDownload video) {
    final event = video.event;
    final reference = NostrEventReference(
      eventId: NostrEventId.parse(event.eventId),
      authorPublicKeyHex: NostrPublicKeyHex.parse(event.authorPublicKeyHex),
      kind: NostrEventKind.parse(event.kind.toInt()),
      identifier: _identifier(event.identifier),
    );
    return VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse(event.eventId),
        creator: _creator(video, event),
        nostrReference: reference,
      ),
      content: _content(video, event),
      metrics: VideoPostMetrics(
        likeCount: _count(video.nostr.likes),
        commentCount: _count(video.nostr.comments),
        viewerHasLiked: false,
      ),
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
  ) {
    final content = event.content.trim();
    return VideoPostContent(
      caption: content.isEmpty ? video.nostr.title : content,
      songName: video.nostr.songName,
      media: _mediaSource(video),
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

  VideoMediaSource _mediaSource(FfiVideoDownload video) {
    final delivery = _delivery(video.nostr.delivery);
    final path = video.localPath?.trim();
    if (delivery == VideoMediaDelivery.progressive &&
        path != null &&
        path.isNotEmpty) {
      return VideoMediaSource.local(path);
    }
    final fallbacks = <String>{video.nostr.url};
    if (delivery == VideoMediaDelivery.progressive) {
      fallbacks.add('$gatewayBaseUrl/video.mp4?id=${video.id}');
    }
    fallbacks.remove(video.url);
    return VideoMediaSource.remote(
      video.url,
      fallbackUrls: fallbacks.toList(growable: false),
      delivery: delivery,
    );
  }

  VideoMediaDelivery _delivery(FfiVideoDelivery delivery) {
    return switch (delivery) {
      FfiVideoDelivery.progressive => VideoMediaDelivery.progressive,
      FfiVideoDelivery.hls => VideoMediaDelivery.hls,
    };
  }
}
