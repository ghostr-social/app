import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('profile metadata cache survives a cache instance round trip', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final profile = ProfileSummary(
      id: ProfileId.parse('npub1nora'),
      displayName: 'Nora Relay',
      handle: '@nora_relay',
      avatarUrl: 'https://cdn.example/nora.png',
    );

    await LocalProfileMetadataCache(preferences).write(profile);
    final restored = await LocalProfileMetadataCache(
      preferences,
    ).read(profile.id);

    expect(restored?.id, profile.id);
    expect(restored?.displayName, 'Nora Relay');
    expect(restored?.handle, '@nora_relay');
    expect(restored?.avatarUrl, 'https://cdn.example/nora.png');
  });
}
