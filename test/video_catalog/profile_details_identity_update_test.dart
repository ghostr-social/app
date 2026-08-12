import 'package:flutter_test/flutter_test.dart';

import '../support/sample_data.dart';

void main() {
  test('profile details reject metadata for a different identity', () {
    final details = sampleProfileDetails();

    expect(
      () => details.withProfile(sampleCreator(id: 'different-profile')),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          'Updated profile identity does not match.',
        ),
      ),
    );
  });
}
