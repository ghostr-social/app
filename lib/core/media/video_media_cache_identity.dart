import 'package:ghostr/core/media/video_media_source.dart';

final class VideoMediaCacheIdentity {
  const VideoMediaCacheIdentity._(this.value);

  factory VideoMediaCacheIdentity.forStorage(VideoMediaSource media) {
    final digest = media.expectedSha256;
    return VideoMediaCacheIdentity._(
      digest == null
          ? 'scope:${_scope(media)}:sources:${_sourceSet(media)}'
          : 'sha256:${digest.value}',
    );
  }

  factory VideoMediaCacheIdentity.forJob(VideoMediaSource media) {
    final digest = media.expectedSha256?.value;
    final prefix = digest == null ? 'scope:${_scope(media)}' : 'sha256:$digest';
    return VideoMediaCacheIdentity._(
      'job:$prefix:import:${_importSource(media)}:${_sourceSet(media)}',
    );
  }

  final String value;

  static String _sourceSet(VideoMediaSource media) {
    final fields = [
      media.remoteDelivery?.name ?? 'none',
      ...media.cacheSourceUrls,
    ];
    return fields.map((field) => '${field.length}:$field').join();
  }

  static String _importSource(VideoMediaSource media) {
    final path = media.importPath;
    return path == null ? 'none' : '${path.length}:$path';
  }

  static String _scope(VideoMediaSource media) {
    final value = media.cacheScope?.value ?? 'unscoped';
    return '${value.length}:$value';
  }

  @override
  bool operator ==(Object other) {
    return other is VideoMediaCacheIdentity && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}

extension VideoMediaCacheIdentitySource on VideoMediaSource {
  VideoMediaCacheIdentity get cacheStorageIdentity {
    return VideoMediaCacheIdentity.forStorage(this);
  }

  VideoMediaCacheIdentity get cacheJobIdentity {
    return VideoMediaCacheIdentity.forJob(this);
  }

  VideoMediaCacheIdentity get inventoryPlaybackIdentity {
    final path = localPath ?? '';
    final proxy = this is ProxiedHlsVideoMediaSource
        ? (this as ProxiedHlsVideoMediaSource).playbackUri.toString()
        : '';
    return VideoMediaCacheIdentity._(
      'playback:${path.length}:$path:${proxy.length}:$proxy:'
      '${cacheJobIdentity.value}',
    );
  }
}
