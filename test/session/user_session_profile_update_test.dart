import 'package:flutter_test/flutter_test.dart';

import '../support/sample_data.dart';

void main() {
  test('session accepts profile metadata only for its own identity', () {
    final session = sampleSession();
    final updated = sampleCreator(
      id: session.profile.id,
      displayName: 'Updated Nora',
    );

    expect(session.withProfile(updated).profile, same(updated));
    expect(
      () => session.withProfile(sampleCreator(id: 'someone-else')),
      throwsStateError,
    );
  });
}
