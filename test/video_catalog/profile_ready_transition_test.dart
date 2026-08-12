import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';

import '../support/sample_data.dart';

void main() {
  test('profile ready transition changes and clears transient fields', () {
    final state = ProfileReady(
      sampleProfileDetails(),
      isRefreshing: true,
      notice: 'old notice',
      refreshError: 'old error',
    );

    final next = state.transition(
      const ProfileReadyTransition(
        isRefreshing: false,
        notice: 'new notice',
        clearRefreshError: true,
      ),
    );

    expect(next.isRefreshing, isFalse);
    expect(next.notice, 'new notice');
    expect(next.refreshError, isNull);
  });
}
