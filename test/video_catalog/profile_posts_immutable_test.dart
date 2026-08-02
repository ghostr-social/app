import 'package:flutter_test/flutter_test.dart';

import '../support/sample_data.dart';

void main() {
  test('does not expose a mutable profile-post collection', () {
    final details = sampleProfileDetails();

    expect(() => details.posts.clear(), throwsUnsupportedError);
  });
}
