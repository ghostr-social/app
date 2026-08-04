import 'package:ghostr/core/media/video_media_metadata.dart';
import 'package:ghostr/core/media/video_sha256.dart';
import 'package:ghostr/core/media/video_media_cache_scope.dart';

part 'expected_sha256_video_media_source.dart';
part 'cached_video_media_source.dart';
part 'importable_video_media_source.dart';
part 'proxied_hls_video_media_source.dart';
part 'proxied_progressive_video_media_source.dart';
part 'scoped_video_media_source.dart';
part 'validated_video_media_source.dart';

enum VideoMediaDelivery { progressive, hls }

const maxVideoCacheSourceCount = 5;

sealed class VideoMediaSource {
  const VideoMediaSource();

  factory VideoMediaSource.local(String rawPath) {
    return LocalVideoMediaSource._(_localPath(rawPath));
  }

  factory VideoMediaSource.cached(
    String rawPath, {
    required String remoteUrl,
    List<String> fallbackUrls = const [],
    VideoMediaDelivery delivery = VideoMediaDelivery.progressive,
  }) {
    return CachedVideoMediaSource._(
      _localPath(rawPath),
      _httpUrl(remoteUrl),
      List<String>.unmodifiable(fallbackUrls.map(_httpUrl)),
      delivery,
    );
  }

  factory VideoMediaSource.importable(
    String rawSourcePath, {
    required String remoteUrl,
    List<String> fallbackUrls = const [],
  }) {
    return ImportableVideoMediaSource._(
      _localPath(rawSourcePath),
      _httpUrl(remoteUrl),
      List<String>.unmodifiable(fallbackUrls.map(_httpUrl)),
    );
  }

  factory VideoMediaSource.remote(
    String rawUrl, {
    List<String> fallbackUrls = const [],
    VideoMediaDelivery delivery = VideoMediaDelivery.progressive,
    VideoMediaMetadata metadata = VideoMediaMetadata.none,
  }) {
    final url = _httpUrl(rawUrl);
    final fallbacks = List<String>.unmodifiable(fallbackUrls.map(_httpUrl));
    return RemoteVideoMediaSource._(url, fallbacks, delivery, metadata);
  }

  factory VideoMediaSource.proxiedHls(String rawUrl) {
    return ProxiedHlsVideoMediaSource(rawUrl);
  }

  factory VideoMediaSource.withExpectedSha256(
    VideoMediaSource source,
    String rawSha256,
  ) {
    if (source.remoteUrl == null) {
      throw const FormatException('A remote video is required.');
    }
    return _ExpectedSha256VideoMediaSource(
      source,
      VideoSha256.parse(rawSha256),
    );
  }

  factory VideoMediaSource.withCacheScope(
    VideoMediaSource source,
    String rawScope,
  ) {
    if (source.remoteUrl == null) {
      throw const FormatException('A remote video is required.');
    }
    return _ScopedVideoMediaSource(
      source,
      VideoMediaCacheScope.parse(rawScope),
    );
  }

  String get debugLabel;

  String? get remoteUrl;

  String? get localPath;

  String? get importPath => null;

  List<String> get fallbackUrls;

  bool get isLocal;

  bool get canCacheAsSingleFile;

  VideoMediaDelivery? get remoteDelivery;

  List<String> get remoteUrls;

  VideoSha256? get expectedSha256 => null;

  VideoMediaCacheScope? get cacheScope => null;

  VideoMediaMetadata get mediaMetadata => switch (this) {
        _ExpectedSha256VideoMediaSource(:final source) => source.mediaMetadata,
        _ScopedVideoMediaSource(:final source) => source.mediaMetadata,
        _ => VideoMediaMetadata.none,
      };

  List<String> get cacheSourceUrls => List<String>.unmodifiable(
        remoteUrls.take(maxVideoCacheSourceCount),
      );
}

final class LocalVideoMediaSource extends VideoMediaSource {
  const LocalVideoMediaSource._(this.path);

  final String path;

  @override
  String get debugLabel => path;

  @override
  List<String> get fallbackUrls => const [];

  @override
  bool get isLocal => true;

  @override
  bool get canCacheAsSingleFile => false;

  @override
  String? get localPath => path;

  @override
  String? get remoteUrl => null;

  @override
  VideoMediaDelivery? get remoteDelivery => null;

  @override
  List<String> get remoteUrls => const [];
}

final class RemoteVideoMediaSource extends VideoMediaSource {
  const RemoteVideoMediaSource._(
    this.url,
    this.fallbackUrls,
    this.delivery,
    this.mediaMetadata,
  );

  final String url;

  @override
  final List<String> fallbackUrls;

  final VideoMediaDelivery delivery;

  @override
  final VideoMediaMetadata mediaMetadata;

  int? get sizeBytes => mediaMetadata.sizeBytes;

  int? get durationMs => mediaMetadata.durationMs;

  @override
  String get debugLabel => url;

  @override
  bool get isLocal => false;

  @override
  bool get canCacheAsSingleFile => delivery == VideoMediaDelivery.progressive;

  @override
  String? get localPath => null;

  @override
  String? get remoteUrl => url;

  @override
  VideoMediaDelivery get remoteDelivery => delivery;

  @override
  List<String> get remoteUrls => List<String>.unmodifiable([
        url,
        ...fallbackUrls,
      ]);
}
