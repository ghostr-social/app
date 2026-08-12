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
  List<String> get remoteUrls =>
      List<String>.unmodifiable([playbackUri.toString()]);
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
  return _isGatewayPlaybackQuery(uri);
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

bool _isGatewayPlaybackQuery(Uri uri) {
  final parameters = uri.queryParametersAll;
  if (parameters.length != 2) return false;
  final id = _singleQueryValue(parameters, 'id');
  final capability = _singleQueryValue(parameters, 'cap');
  return id != null &&
      capability != null &&
      _isGatewayPostId(id) &&
      _isGatewayCapability(capability);
}

String? _singleQueryValue(Map<String, List<String>> parameters, String key) {
  final values = parameters[key];
  return values?.length == 1 ? values!.single : null;
}

bool _isGatewayPostId(String value) {
  return value.isNotEmpty && _gatewayPostIdPattern.hasMatch(value);
}

final _gatewayPostIdPattern = RegExp(r'^[A-Za-z0-9_-]+$');
final _gatewayCapabilityPattern = RegExp(r'^[A-Za-z0-9_-]{43}$');

bool _isGatewayCapability(String value) {
  return _gatewayCapabilityPattern.hasMatch(value);
}
