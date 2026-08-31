import 'dart:convert';

final class WarpOfflineRestartManifest {
  const WarpOfflineRestartManifest({
    required this.eventId,
    required this.originPort,
    required this.relay,
    required this.viewerPublicKey,
  });

  factory WarpOfflineRestartManifest.decode(String encoded) {
    final value = jsonDecode(encoded);
    if (value is! Map<String, Object?>) {
      throw const FormatException('Invalid WARP offline restart manifest.');
    }
    return WarpOfflineRestartManifest(
      eventId: _string(value, 'event_id'),
      originPort: _positiveInt(value, 'origin_port'),
      relay: Uri.parse(_string(value, 'relay')),
      viewerPublicKey: _string(value, 'viewer_public_key'),
    );
  }

  final String eventId;
  final int originPort;
  final Uri relay;
  final String viewerPublicKey;

  String encode() => jsonEncode({
    'event_id': eventId,
    'origin_port': originPort,
    'relay': relay.toString(),
    'viewer_public_key': viewerPublicKey,
  });
}

String _string(Map<String, Object?> value, String key) {
  final result = value[key];
  if (result is! String || result.isEmpty) {
    throw FormatException('Invalid WARP offline restart $key.');
  }
  return result;
}

int _positiveInt(Map<String, Object?> value, String key) {
  final result = value[key];
  if (result is! int || result <= 0) {
    throw FormatException('Invalid WARP offline restart $key.');
  }
  return result;
}
