import 'package:flutter_test/flutter_test.dart';

import '../support/sample_data.dart';

void main() {
  test('preserves an omitted block state when profile details are copied', () {
    final original = sampleProfileDetails();

    final changed = original.copyWith(isFollowing: true);

    expect(changed.isFollowing, isTrue);
    expect(changed.isBlocked, original.isBlocked);
  });
}
