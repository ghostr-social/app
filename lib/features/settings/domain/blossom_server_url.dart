import 'package:ghostr/core/network/secure_endpoint_policy.dart';

class BlossomServerUrl {
  const BlossomServerUrl._(this.value);

  factory BlossomServerUrl.parse(String raw) {
    final parsed = tryParse(raw);
    if (parsed == null) {
      throw const FormatException('Enter a valid HTTPS Blossom server URL.');
    }
    return parsed;
  }

  static BlossomServerUrl? tryParse(String raw) {
    final value = _policy.normalize(raw);
    return value == null ? null : BlossomServerUrl._(value);
  }

  static const _policy = SecureEndpointPolicy(
    secureScheme: 'https',
    localDevelopmentScheme: 'http',
  );

  final String value;

  @override
  bool operator ==(Object other) {
    return other is BlossomServerUrl && other.value == value;
  }

  @override
  int get hashCode => value.hashCode;
}
