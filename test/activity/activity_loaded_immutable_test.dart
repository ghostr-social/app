import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/presentation/activity_cubit.dart';

import '../support/sample_data.dart';

void main() {
  test('does not expose a mutable loaded activity collection', () {
    final state = ActivityLoaded([sampleActivity()]);

    expect(() => state.items.clear(), throwsUnsupportedError);
  });
}
