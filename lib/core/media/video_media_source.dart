enum VideoMediaDelivery { progressive, hls }

sealed class VideoMediaSource {
  const VideoMediaSource();

  factory VideoMediaSource.local(String rawPath) {
    final path = rawPath.trim();
    if (path.isEmpty) throw const FormatException('Video path is required.');
    return LocalVideoMediaSource._(path);
  }

  factory VideoMediaSource.remote(
    String rawUrl, {
    List<String> fallbackUrls = const [],
    VideoMediaDelivery delivery = VideoMediaDelivery.progressive,
  }) {
    final url = _httpUrl(rawUrl);
    final fallbacks = List<String>.unmodifiable(fallbackUrls.map(_httpUrl));
    return RemoteVideoMediaSource._(url, fallbacks, delivery);
  }

  String get debugLabel;

  String? get remoteUrl;

  String? get localPath;

  List<String> get fallbackUrls;

  bool get isLocal;

  bool get canCacheAsSingleFile;

  VideoMediaDelivery? get remoteDelivery;

  List<String> get remoteUrls;
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
  );

  final String url;

  @override
  final List<String> fallbackUrls;

  final VideoMediaDelivery delivery;

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

String _httpUrl(String raw) {
  final value = raw.trim();
  final uri = Uri.tryParse(value);
  final isHttp = uri?.scheme == 'https' || uri?.scheme == 'http';
  if (uri == null || !isHttp || uri.host.isEmpty) {
    throw FormatException('Invalid video URL: $raw');
  }
  return value;
}
