part of 'video_media_source.dart';

final class _ScopedVideoMediaSource extends VideoMediaSource {
  const _ScopedVideoMediaSource(this.source, this.cacheScope);

  final VideoMediaSource source;

  @override
  final VideoMediaCacheScope cacheScope;

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
  VideoSha256? get expectedSha256 => source.expectedSha256;
}
