import 'package:ghostr/features/app_update/domain/android_abi.dart';

final class GitHubReleaseUriPolicy {
  const GitHubReleaseUriPolicy();

  Uri parseReleasePage(String raw, String versionName) {
    final expected =
        'https://github.com/ghostr-social/app/releases/tag/v$versionName';
    return _parseExact(raw, expected);
  }

  Uri parseArtifact(String raw, String versionName, AndroidAbi abi) {
    final name = 'ghostr-v$versionName-${abi.value}.apk';
    final expected =
        'https://github.com/ghostr-social/app/releases/download/'
        'v$versionName/$name';
    return _parseExact(raw, expected);
  }

  Uri _parseExact(String raw, String expected) {
    final uri = Uri.tryParse(raw);
    if (uri == null || raw != expected) {
      throw const FormatException('A trusted GitHub release URL is required.');
    }
    return uri;
  }
}
