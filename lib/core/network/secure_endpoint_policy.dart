class SecureEndpointPolicy {
  const SecureEndpointPolicy({
    required this.secureScheme,
    required this.localDevelopmentScheme,
  });

  static const _localHosts = {'localhost', '127.0.0.1', '::1'};

  final String secureScheme;
  final String localDevelopmentScheme;

  String? normalize(String raw) {
    final uri = Uri.tryParse(raw.trim());
    if (uri == null || !_isSupported(uri)) return null;
    return _withoutTrailingSlash(uri);
  }

  bool _isSupported(Uri uri) {
    return _usesAllowedScheme(uri) &&
        uri.host.isNotEmpty &&
        !uri.hasQuery &&
        !uri.hasFragment &&
        uri.userInfo.isEmpty;
  }

  bool _usesAllowedScheme(Uri uri) {
    if (uri.scheme == secureScheme) return true;
    return uri.scheme == localDevelopmentScheme &&
        _localHosts.contains(uri.host);
  }

  String _withoutTrailingSlash(Uri uri) {
    final value = uri.toString();
    return uri.path == '/' ? value.substring(0, value.length - 1) : value;
  }
}
