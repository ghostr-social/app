import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/domain/profile_details.dart';

void main() {
  test('rejects a current-user relationship that is also followed', () {
    expect(
      () => ProfileRelationship(
        isFollowing: true,
        isBlocked: false,
        isCurrentUser: true,
      ),
      throwsStateError,
    );
  });
}
