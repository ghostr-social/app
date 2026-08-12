import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test(
    'profile metadata cache translates corrupt storage to AppFailure',
    () async {
      SharedPreferences.setMockInitialValues({});
      final preferences = await SharedPreferences.getInstance();
      final cache = LocalProfileMetadataCache(preferences);
      final profile = ProfileSummary(
        id: ProfileId.parse('npub1nora'),
        displayName: 'Nora',
        handle: '@nora',
        avatarUrl: null,
      );
      await cache.write(profile);
      await preferences.setString(preferences.getKeys().single, '{not-json');

      await expectLater(cache.read(profile.id), throwsA(isA<AppFailure>()));
    },
  );
}
