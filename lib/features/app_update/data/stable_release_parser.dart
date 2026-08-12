import 'dart:convert';

import 'package:ghostr/features/app_update/data/github_release_uri_policy.dart';
import 'package:ghostr/features/app_update/domain/android_abi.dart';
import 'package:ghostr/features/app_update/domain/android_version_code.dart';
import 'package:ghostr/features/app_update/domain/release_artifact.dart';
import 'package:ghostr/features/app_update/domain/stable_release.dart';
import 'package:ghostr/features/app_update/domain/update_package_sha256.dart';

final class StableReleaseParser {
  const StableReleaseParser({
    GitHubReleaseUriPolicy uriPolicy = const GitHubReleaseUriPolicy(),
  }) : _uriPolicy = uriPolicy;

  static const _fields = {
    'schemaVersion',
    'namespace',
    'packageName',
    'channel',
    'versionName',
    'versionCode',
    'publishedAt',
    'releaseUrl',
    'artifacts',
  };
  static const _artifactFields = {'abi', 'url', 'size', 'sha256'};
  static final _versionPattern = RegExp(r'^[0-9]+\.[0-9]+\.[0-9]+$');
  static final _timestampPattern = RegExp(
    r'^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z$',
  );

  final GitHubReleaseUriPolicy _uriPolicy;

  StableRelease parse(String source) {
    final json = _decodeObject(source);
    _expectFields(json, _fields);
    _expectIdentity(json);
    final versionName = _parseVersionName(json['versionName']);
    return StableRelease(
      versionName: versionName,
      versionCode: _parseVersionCode(json['versionCode']),
      publishedAt: _parseTimestamp(json['publishedAt']),
      releaseUri: _uriPolicy.parseReleasePage(
        _readString(json['releaseUrl']),
        versionName,
      ),
      artifacts: _parseArtifacts(json['artifacts'], versionName),
    );
  }

  Map<String, Object?> _decodeObject(String source) {
    try {
      final decoded = jsonDecode(source);
      if (decoded is! Map<String, Object?>) throw const FormatException();
      return decoded;
    } on FormatException {
      throw const FormatException('Stable release metadata is malformed.');
    }
  }

  void _expectIdentity(Map<String, Object?> json) {
    if (json['schemaVersion'] != 1 ||
        json['namespace'] != 'ghostr.social' ||
        json['packageName'] != 'app.ghostr' ||
        json['channel'] != 'stable') {
      throw const FormatException('Stable release identity is invalid.');
    }
  }

  String _parseVersionName(Object? raw) {
    final versionName = _readString(raw);
    if (!_versionPattern.hasMatch(versionName)) {
      throw const FormatException('Release version name is invalid.');
    }
    return versionName;
  }

  AndroidVersionCode _parseVersionCode(Object? raw) {
    if (raw is! int || raw < 1 || raw > AndroidVersionCode.maximum) {
      throw const FormatException('Release version code is invalid.');
    }
    return AndroidVersionCode(raw);
  }

  DateTime _parseTimestamp(Object? raw) {
    final source = _readString(raw);
    if (!_timestampPattern.hasMatch(source)) {
      throw const FormatException('Timestamp is invalid.');
    }
    final value = DateTime.tryParse(source);
    final canonical = '${source.substring(0, source.length - 1)}.000Z';
    if (value == null || value.toIso8601String() != canonical) {
      throw const FormatException('Timestamp is invalid.');
    }
    return value;
  }

  Map<AndroidAbi, ReleaseArtifact> _parseArtifacts(
    Object? raw,
    String versionName,
  ) {
    if (raw is! List<Object?> || raw.isEmpty) {
      throw const FormatException('Release artifacts are required.');
    }
    final artifacts = <AndroidAbi, ReleaseArtifact>{};
    for (final value in raw) {
      final artifact = _parseArtifact(value, versionName);
      if (artifacts.containsKey(artifact.abi)) {
        throw const FormatException('Release artifact ABI is duplicated.');
      }
      artifacts[artifact.abi] = artifact;
    }
    return artifacts;
  }

  ReleaseArtifact _parseArtifact(Object? raw, String versionName) {
    if (raw is! Map<String, Object?>) {
      throw const FormatException('Release artifact is malformed.');
    }
    _expectFields(raw, _artifactFields);
    final abi = AndroidAbi.tryParse(_readString(raw['abi']));
    final size = raw['size'];
    final digest = UpdatePackageSha256.tryParse(_readString(raw['sha256']));
    if (abi == null ||
        size is! int ||
        size < 1 ||
        size > ReleaseArtifact.maximumSizeBytes ||
        digest == null) {
      throw const FormatException('Release artifact values are invalid.');
    }
    return ReleaseArtifact(
      abi: abi,
      uri: _uriPolicy.parseArtifact(_readString(raw['url']), versionName, abi),
      sizeBytes: size,
      sha256: digest,
    );
  }

  String _readString(Object? raw) {
    if (raw is! String) throw const FormatException('A string is required.');
    return raw;
  }

  void _expectFields(Map<String, Object?> json, Set<String> fields) {
    if (json.length != fields.length || !fields.every(json.containsKey)) {
      throw const FormatException('Release metadata fields are invalid.');
    }
  }
}
