part of 'video_media_source.dart';

final class ProxiedHlsVideoMediaSource extends VideoMediaSource {
  factory ProxiedHlsVideoMediaSource(String rawUrl) {
    return ProxiedHlsVideoMediaSource._(_trustedHlsGatewayUri(rawUrl));
  }

  const ProxiedHlsVideoMediaSource._(this.playbackUri);

  final Uri playbackUri;

  @override
  String get debugLabel => 'Secure HLS stream';

  @override
  List<String> get fallbackUrls => const [];

  @override
  bool get isLocal => false;

  @override
  bool get canCacheAsSingleFile => false;

  @override
  String? get localPath => null;

  @override
  String? get remoteUrl => null;

  @override
  VideoMediaDelivery get remoteDelivery => VideoMediaDelivery.hls;

  @override
  List<String> get remoteUrls => const [];
}

Uri _trustedHlsGatewayUri(String raw) {
  final uri = Uri.tryParse(raw.trim());
  if (uri == null || !_isTrustedHlsGatewayUri(uri)) {
    throw FormatException('Invalid secure HLS gateway URL: $raw');
  }
  return uri;
}

bool _isTrustedHlsGatewayUri(Uri uri) {
  final parts = uri.pathSegments;
  return uri.scheme == 'http' &&
      _isLiteralLoopback(uri.host) &&
      uri.hasPort &&
      uri.userInfo.isEmpty &&
      !uri.hasQuery &&
      !uri.hasFragment &&
      parts.length == 3 &&
      parts.first == 'hls' &&
      _isHlsSessionId(parts[1]) &&
      parts.last == 'index.m3u8';
}

bool _isLiteralLoopback(String host) => host == '127.0.0.1' || host == '::1';

bool _isHlsSessionId(String value) {
  return value.length == 64 &&
      value.codeUnits.every(
        (unit) =>
            unit >= 48 && unit <= 57 ||
            unit >= 65 && unit <= 70 ||
            unit >= 97 && unit <= 102,
      );
}
