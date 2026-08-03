import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/video_catalog/presentation/profile_state.dart';

import '../support/sample_data.dart';

void main() {
  test('profile states expose data only for their active variant', () {
    final loading = ProfileState.loading();
    final failure = ProfileState.failure('offline');
    final ready = ProfileState.ready(sampleProfileDetails()) as ProfileReady;

    expect(loading.status, ProfileStatus.loading);
    expect(loading.details, isNull);
    expect(loading.isUpdating, isFalse);
    expect(loading.message, isNull);
    expect(loading.notice, isNull);
    expect(failure.status, ProfileStatus.failure);
    expect(failure.message, 'offline');
    expect(ready.status, ProfileStatus.ready);
    expect(ready.details, isNotNull);
    expect(ready.withoutNotice().notice, isNull);
  });
}
