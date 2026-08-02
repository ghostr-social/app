import 'package:ghostr/core/network/secure_endpoint_policy.dart';

class RelayUrl {
  const RelayUrl._(this.value);

  factory RelayUrl.parse(String raw) {
    final parsed = tryParse(raw);
    if (parsed == null) {
      throw const FormatException('Enter a valid ws:// or wss:// relay URL.');
    }
    return parsed;
  }

  static RelayUrl? tryParse(String raw) {
    final value = _policy.normalize(raw);
    return value == null ? null : RelayUrl._(value);
  }

  static const _policy = SecureEndpointPolicy(
    secureScheme: 'wss',
    localDevelopmentScheme: 'ws',
  );

  final String value;

  @override
  bool operator ==(Object other) => other is RelayUrl && other.value == value;

  @override
  int get hashCode => value.hashCode;
}
