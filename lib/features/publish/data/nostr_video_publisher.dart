import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/core/media/selected_media.dart';
import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/core/nostr/nostr_event_client.dart';
import 'package:ghostr/core/nostr/nostr_event_record.dart';
import 'package:ghostr/core/time/clock.dart';
import 'package:ghostr/features/publish/domain/nostr_video_publisher_port.dart';
import 'package:ghostr/features/publish/domain/video_media_upload_port.dart';
import 'package:ghostr/features/publish/domain/uploaded_video_media.dart';
import 'package:ghostr/features/session/domain/user_session.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

class NostrVideoPublisher implements NostrVideoPublisherPort {
  const NostrVideoPublisher({
    required NostrEventClient eventClient,
    required VideoMediaUploadPort mediaUploader,
    Clock clock = systemClock,
  })  : _eventClient = eventClient,
        _mediaUploader = mediaUploader,
        _clock = clock;

  final NostrEventClient _eventClient;
  final VideoMediaUploadPort _mediaUploader;
  final Clock _clock;

  @override
  Future<VideoPost> publish({
    required UserSession session,
    required SelectedMedia media,
    required String caption,
  }) async {
    _verifyIdentity(session);
    final authorPublicKeyHex = session.identity.publicKeyHex;
    final uploaded = await _mediaUploader.upload(media);
    _verifyIdentity(session);
    final publishedAt = _clock().toUtc();
    final title = caption.trim().isEmpty ? media.label : caption.trim();
    final eventId = await _eventClient.publish(
      NostrUnsignedEvent(
        kind: 22,
        tags: _videoTags(uploaded, title, publishedAt),
        content: title,
      ),
      expectedAuthor: authorPublicKeyHex,
    );
    final id = VideoPostId.parse(eventId.value);
    return _toPost(
      session,
      uploaded,
      authorPublicKeyHex,
      _PublishedVideo(
        selected: media,
        caption: title,
        id: id,
        publishedAt: publishedAt,
      ),
    );
  }

  List<List<String>> _videoTags(
    UploadedVideoMedia media,
    String title,
    DateTime publishedAt,
  ) {
    return <List<String>>[
      <String>['title', title],
      <String>['published_at', '${publishedAt.millisecondsSinceEpoch ~/ 1000}'],
      <String>['alt', title],
      <String>[
        'imeta',
        'url ${media.primaryUrl}',
        ...media.fallbackUrls.map((url) => 'fallback $url'),
        'm ${media.mimeType}',
        'x ${media.sha256}',
        'size ${media.sizeBytes}',
      ],
    ];
  }

  VideoPost _toPost(
    UserSession session,
    UploadedVideoMedia uploaded,
    NostrPublicKeyHex authorPublicKeyHex,
    _PublishedVideo video,
  ) {
    return VideoPost(
      identity: VideoPostIdentity(
        id: video.id,
        creator: session.profile,
        nostrReference: NostrEventReference(
          eventId: NostrEventId.parse(video.id.value),
          authorPublicKeyHex: authorPublicKeyHex,
          kind: NostrEventKind.parse(22),
        ),
      ),
      content: VideoPostContent(
        caption: video.caption,
        songName: video.selected.label,
        media: _remoteMedia(uploaded, video.id.value),
        publishedAt: video.publishedAt,
      ),
      metrics: VideoPostMetrics(
        likeCount: 0,
        commentCount: 0,
        viewerHasLiked: false,
      ),
    );
  }

  VideoMediaSource _remoteMedia(UploadedVideoMedia uploaded, String eventId) {
    final source = VideoMediaSource.remote(
      uploaded.primaryUrl,
      fallbackUrls: uploaded.fallbackUrls,
    );
    final verified =
        VideoMediaSource.withExpectedSha256(source, uploaded.sha256);
    return VideoMediaSource.withCacheScope(verified, eventId);
  }

  void _verifyIdentity(UserSession session) {
    if (session.identity.publicKeyHex != _eventClient.publicKeyHex) {
      throw const AppFailure(
          'The active Nostr signer does not match the session.');
    }
  }
}

class _PublishedVideo {
  const _PublishedVideo({
    required this.selected,
    required this.caption,
    required this.id,
    required this.publishedAt,
  });

  final SelectedMedia selected;
  final String caption;
  final VideoPostId id;
  final DateTime publishedAt;
}
