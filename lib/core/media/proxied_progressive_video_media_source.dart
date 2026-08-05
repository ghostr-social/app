part of 'video_media_source.dart';

final class ProxiedProgressiveVideoMediaSource extends VideoMediaSource {
  factory ProxiedProgressiveVideoMediaSource(String rawUrl) {
    return ProxiedProgressiveVideoMediaSource._(
      _trustedProgressiveGatewayUri(rawUrl),
    );
  }

  const ProxiedProgressiveVideoMediaSource._(this.playbackUri);

  final Uri playbackUri;

  @override
  String get debugLabel => 'Progressive loopback stream';

  @override
  List<String> get fallbackUrls => const [];

  @override
  bool get isLocal => false;

  @override
  bool get canCacheAsSingleFile => false;

  @override
  String? get localPath => null;

  @override
  String? get remoteUrl => playbackUri.toString();

  @override
  VideoMediaDelivery get remoteDelivery => VideoMediaDelivery.progressive;

  @override
  List<String> get remoteUrls => List<String>.unmodifiable([
        playbackUri.toString(),
      ]);
}

Uri _trustedProgressiveGatewayUri(String raw) {
  final uri = Uri.tryParse(raw.trim());
  if (uri == null || !_isTrustedProgressiveGatewayUri(uri)) {
    throw FormatException('Invalid progressive gateway URL: $raw');
  }
  return uri;
}

bool _isTrustedProgressiveGatewayUri(Uri uri) {
  if (!_isLoopbackHttp(uri)) return false;
  if (!_isSafeGatewayLocation(uri)) return false;
  return _isSinglePostIdQuery(uri);
}

bool _isLoopbackHttp(Uri uri) {
  return uri.scheme == 'http' && _isLiteralLoopback(uri.host);
}

bool _isSafeGatewayLocation(Uri uri) {
  return uri.hasPort &&
      uri.userInfo.isEmpty &&
      !uri.hasFragment &&
      uri.path == '/video.mp4';
}

bool _isSinglePostIdQuery(Uri uri) {
  final parameters = uri.queryParametersAll;
  if (parameters.length != 1) return false;
  final ids = parameters['id'];
  return ids != null && ids.length == 1 && _isGatewayPostId(ids.single);
}

bool _isGatewayPostId(String value) {
  return value.isNotEmpty && _gatewayPostIdPattern.hasMatch(value);
}

final _gatewayPostIdPattern = RegExp(r'^[A-Za-z0-9_-]+$');
