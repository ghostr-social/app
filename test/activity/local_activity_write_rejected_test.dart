import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:ghostr/features/activity/domain/activity_type.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _Preferences extends Mock implements SharedPreferences {}

void main() {
  test('rejects activity recording when preferences refuse the write',
      () async {
    final preferences = _Preferences();
    when(() => preferences.getString(any())).thenReturn(null);
    when(() => preferences.setString(any(), any()))
        .thenAnswer((_) async => false);
    final repository = LocalActivityRepository(preferences);
    final item = ActivityItem(
      id: ActivityId.parse('activity-1'),
      type: ActivityType.like,
      description: ActivityDescription(
        title: 'Ada',
        body: 'liked your video',
      ),
      occurredAt: DateTime.utc(2026),
    );

    await expectLater(repository.record(item), throwsA(isA<AppFailure>()));
  });
}
