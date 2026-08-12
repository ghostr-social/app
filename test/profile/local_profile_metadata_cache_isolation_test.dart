import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/profile/data/local_profile_metadata_cache.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  test('profile metadata cache isolates entries by profile id', () async {
    SharedPreferences.setMockInitialValues({});
    final preferences = await SharedPreferences.getInstance();
    final cache = LocalProfileMetadataCache(preferences);
    final noraId = ProfileId.parse('npub1nora');
    final eliId = ProfileId.parse('npub1eli');

    await cache.write(
      ProfileSummary(
        id: noraId,
        displayName: 'Nora',
        handle: '@nora',
        avatarUrl: null,
      ),
    );
    expect(await cache.read(eliId), isNull);
    await cache.write(
      ProfileSummary(
        id: eliId,
        displayName: 'Eli',
        handle: '@eli',
        avatarUrl: null,
      ),
    );

    expect((await cache.read(noraId))?.displayName, 'Nora');
    expect((await cache.read(eliId))?.displayName, 'Eli');
    expect(preferences.getKeys(), hasLength(2));
    expect(
      preferences.getKeys().any((key) => key.endsWith(noraId.value)),
      isTrue,
    );
    expect(
      preferences.getKeys().any((key) => key.endsWith(eliId.value)),
      isTrue,
    );
  });
}
