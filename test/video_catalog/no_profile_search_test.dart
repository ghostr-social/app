import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/data/nostr_profile_search_port.dart';

void main() {
  test('the null profile search finds nobody', () async {
    expect(
      await const NoNostrProfileSearch().searchProfiles('anyone'),
      isEmpty,
    );
  });
}
