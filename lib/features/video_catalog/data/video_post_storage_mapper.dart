import 'package:ghostr/core/media/video_media_source.dart';
import 'package:ghostr/features/video_catalog/domain/nostr_event_reference.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/video_post.dart';
import 'package:ghostr/features/video_catalog/domain/video_post_id.dart';

class VideoPostStorageMapper {
  const VideoPostStorageMapper();

  VideoPost fromMap(Map<String, dynamic> map) {
    return VideoPost(
      identity: VideoPostIdentity(
        id: VideoPostId.parse(_required<String>(map, 'id')),
        creator: _profileFromMap(_requiredMap(map, 'creator')),
        nostrReference: _referenceFromValue(map['nostrReference']),
      ),
      content: VideoPostContent(
        caption: _required<String>(map, 'caption'),
        songName: _required<String>(map, 'songName'),
        media: _mediaFromMap(_requiredMap(map, 'media')),
        publishedAt: DateTime.parse(_required<String>(map, 'publishedAt')),
      ),
      metrics: VideoPostMetrics(
        likeCount: _required<int>(map, 'likeCount'),
        commentCount: _required<int>(map, 'commentCount'),
        viewerHasLiked: _optional<bool>(map, 'viewerHasLiked') ?? false,
      ),
    );
  }

  Map<String, Object?> toMap(VideoPost post) {
    return <String, Object?>{
      'id': post.id.value,
      'creator': _profileToMap(post.creator),
      'caption': post.caption,
      'songName': post.songName,
      'likeCount': post.likeCount,
      'commentCount': post.commentCount,
      'viewerHasLiked': post.viewerHasLiked,
      'media': _mediaToMap(post.media),
      'publishedAt': post.publishedAt.toIso8601String(),
      'nostrReference': _referenceToMap(post.nostrReference),
    };
  }

  ProfileSummary _profileFromMap(Map<String, dynamic> map) {
    return ProfileSummary(
      id: ProfileId.parse(_required<String>(map, 'id')),
      displayName: _required<String>(map, 'displayName'),
      handle: _required<String>(map, 'handle'),
      avatarUrl: _optional<String>(map, 'avatarUrl'),
    );
  }

  Map<String, Object?> _profileToMap(ProfileSummary profile) {
    return <String, Object?>{
      'id': profile.id.value,
      'displayName': profile.displayName,
      'handle': profile.handle,
      'avatarUrl': profile.avatarUrl,
    };
  }

  VideoMediaSource _mediaFromMap(Map<String, dynamic> map) {
    final localPath = _optional<String>(map, 'localPath');
    if (localPath != null) return VideoMediaSource.local(localPath);
    return VideoMediaSource.remote(
      _required<String>(map, 'remoteUrl'),
      fallbackUrls: _stringList(map, 'fallbackUrls'),
    );
  }

  Map<String, Object?> _mediaToMap(VideoMediaSource media) {
    return <String, Object?>{
      'localPath': media.localPath,
      'remoteUrl': media.remoteUrl,
      'fallbackUrls': media.fallbackUrls,
    };
  }

  NostrEventReference? _referenceFromValue(Object? value) {
    if (value == null) return null;
    if (value is! Map<String, dynamic>) {
      throw const FormatException('Invalid Nostr event reference.');
    }
    return NostrEventReference(
      eventId: NostrEventId.parse(_required<String>(value, 'eventId')),
      authorPublicKeyHex: NostrPublicKeyHex.parse(
        _required<String>(value, 'authorPublicKeyHex'),
      ),
      kind: NostrEventKind.parse(_required<int>(value, 'kind')),
      identifier: _referenceIdentifier(value),
    );
  }

  Map<String, Object?>? _referenceToMap(NostrEventReference? reference) {
    if (reference == null) return null;
    return <String, Object?>{
      'eventId': reference.eventId.value,
      'authorPublicKeyHex': reference.authorPublicKeyHex.value,
      'kind': reference.kind.value,
      'identifier': reference.identifier?.value,
    };
  }

  NostrEventIdentifier? _referenceIdentifier(Map<String, dynamic> map) {
    final value = _optional<String>(map, 'identifier');
    return value == null ? null : NostrEventIdentifier.parse(value);
  }

  Map<String, dynamic> _requiredMap(
    Map<String, dynamic> map,
    String key,
  ) {
    final value = map[key];
    if (value is Map<String, dynamic>) return value;
    throw FormatException('Video post field "$key" must be an object.');
  }

  List<String> _stringList(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value == null) return const <String>[];
    if (value is List<dynamic> && value.every((item) => item is String)) {
      return value.cast<String>();
    }
    throw FormatException('Video post field "$key" must be a string list.');
  }

  T _required<T>(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value is T) return value;
    throw FormatException('Video post field "$key" has an invalid type.');
  }

  T? _optional<T>(Map<String, dynamic> map, String key) {
    final value = map[key];
    if (value == null) return null;
    if (value is T) return value;
    throw FormatException('Video post field "$key" has an invalid type.');
  }
}
