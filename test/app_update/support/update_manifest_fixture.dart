import 'dart:convert';

Map<String, Object?> stableManifestMap({
  String versionName = '1.2.3',
  int versionCode = 1002003,
  String publishedAt = '2026-08-11T12:00:00Z',
  Object? releaseUrl,
  List<Object?>? artifacts,
}) {
  return <String, Object?>{
    'schemaVersion': 1,
    'namespace': 'ghostr.social',
    'packageName': 'app.ghostr',
    'channel': 'stable',
    'versionName': versionName,
    'versionCode': versionCode,
    'publishedAt': publishedAt,
    'releaseUrl':
        releaseUrl ??
        'https://github.com/ghostr-social/app/releases/tag/v$versionName',
    'artifacts':
        artifacts ??
        [
          stableArtifactMap('arm64-v8a', versionName: versionName),
          stableArtifactMap('armeabi-v7a', versionName: versionName),
          stableArtifactMap('x86_64', versionName: versionName),
        ],
  };
}

Map<String, Object?> stableArtifactMap(
  String abi, {
  String versionName = '1.2.3',
  Object? url,
  Object? size = 4,
  Object? sha256,
}) {
  return <String, Object?>{
    'abi': abi,
    'url':
        url ??
        'https://github.com/ghostr-social/app/releases/download/'
            'v$versionName/ghostr-v$versionName-$abi.apk',
    'size': size,
    'sha256': sha256 ?? 'a' * 64,
  };
}

String stableManifestJson([Map<String, Object?>? value]) {
  return jsonEncode(value ?? stableManifestMap());
}
