part of 'video_media_source.dart';

final class ImportableVideoMediaSource extends VideoMediaSource {
  const ImportableVideoMediaSource._(
    this.sourcePath,
    this.url,
    this.fallbackUrls,
  );

  final String sourcePath;
  final String url;
  @override
  final List<String> fallbackUrls;

  @override
  String get debugLabel => url;

  @override
  String get importPath => sourcePath;

  @override
  bool get isLocal => false;

  @override
  bool get canCacheAsSingleFile => true;

  @override
  String? get localPath => null;

  @override
  String get remoteUrl => url;

  @override
  VideoMediaDelivery get remoteDelivery => VideoMediaDelivery.progressive;

  @override
  List<String> get remoteUrls => List<String>.unmodifiable([
        url,
        ...fallbackUrls,
      ]);
}
