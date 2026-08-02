import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';
import 'package:ghostr/features/video_catalog/domain/profile_summary.dart';
import 'package:ghostr/features/video_catalog/domain/profile_id.dart';
import '../support/sample_data.dart';

void main() {
  test('builds profile display values', () {
    final profile = sampleCreator(displayName: 'Nora Relay');

    expect(profile.initials, 'NR');
    expect(
        ProfileDetails.empty(ProfileSummary.unknown(ProfileId.parse('missing')))
            .posts,
        isEmpty);
  });
}
