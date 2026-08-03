part of 'video_media_source.dart';

final class _ExpectedSha256VideoMediaSource extends VideoMediaSource {
  const _ExpectedSha256VideoMediaSource(this.source, this.expectedSha256);

  final VideoMediaSource source;

  @override
  final VideoSha256 expectedSha256;

  @override
  String get debugLabel => source.debugLabel;

  @override
  String? get remoteUrl => source.remoteUrl;

  @override
  String? get localPath => source.localPath;

  @override
  String? get importPath => source.importPath;

  @override
  List<String> get fallbackUrls => source.fallbackUrls;

  @override
  bool get isLocal => source.isLocal;

  @override
  bool get canCacheAsSingleFile => source.canCacheAsSingleFile;

  @override
  VideoMediaDelivery? get remoteDelivery => source.remoteDelivery;

  @override
  List<String> get remoteUrls => source.remoteUrls;

  @override
  VideoMediaCacheScope? get cacheScope => source.cacheScope;
}
