part of 'video_media_source.dart';

final class CachedVideoMediaSource extends VideoMediaSource {
  const CachedVideoMediaSource._(
    this.path,
    this.url,
    this.fallbackUrls,
    this.delivery,
  );

  final String path;
  final String url;
  @override
  final List<String> fallbackUrls;
  final VideoMediaDelivery delivery;

  @override
  String get debugLabel => path;

  @override
  bool get isLocal => true;

  @override
  bool get canCacheAsSingleFile => false;

  @override
  String get localPath => path;

  @override
  String get remoteUrl => url;

  @override
  VideoMediaDelivery get remoteDelivery => delivery;

  @override
  List<String> get remoteUrls => List<String>.unmodifiable([
        url,
        ...fallbackUrls,
      ]);
}
