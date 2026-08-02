import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/sample_data.dart';

void main() {
  test('persists activity and returns the newest item first', () async {
    SharedPreferences.setMockInitialValues({});
    final repository = LocalActivityRepository(
      await SharedPreferences.getInstance(),
    );
    final older = sampleActivity();
    final newer = ActivityItem(
      id: ActivityId.parse('activity-2'),
      type: ActivityType.follow,
      description: ActivityDescription(
        title: 'Followed a creator',
        body: 'Newer activity',
      ),
      occurredAt: DateTime(2026, 3, 13),
    );

    expect(await repository.load(), isEmpty);
    await repository.record(older);
    await repository.record(newer);

    expect((await repository.load()).map((item) => item.id), [
      'activity-2',
      'activity-1',
    ]);
  });
}
