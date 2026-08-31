import 'dart:convert';
import 'dart:io';

import 'warp_offline_restart_manifest.dart';

bool warpOfflineSnapshotCommitted(
  File file,
  WarpOfflineRestartManifest manifest,
) {
  if (!file.existsSync()) return false;
  try {
    final root = jsonDecode(file.readAsStringSync());
    if (root is! Map<String, Object?> || root['version'] != 1) return false;
    return _matchesViewer(root['viewer'], manifest.viewerPublicKey) &&
        _containsEvent(root['events'], manifest.eventId);
  } on FileSystemException {
    return false;
  } on FormatException {
    return false;
  }
}

bool _matchesViewer(Object? value, String publicKey) {
  return value is Map<String, Object?> &&
      value['scope'] == 'signed_in' &&
      value['public_key'] == publicKey;
}

bool _containsEvent(Object? value, String eventId) {
  if (value is! List<Object?>) return false;
  return value.any((item) {
    return item is Map<String, Object?> && item['id'] == eventId;
  });
}
