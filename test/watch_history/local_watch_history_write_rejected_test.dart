import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/features/watch_history/data/local_watch_history_repository.dart';
import 'package:ghostr/features/watch_history/domain/watch_history_entry.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/test_account_storage_scope.dart';

class _Preferences extends Mock implements SharedPreferences {}

void main() {
  test('rejects watch recording when preferences refuse the write', () async {
    final preferences = _Preferences();
    when(() => preferences.getString(any())).thenReturn(null);
    when(() => preferences.setString(any(), any()))
        .thenAnswer((_) async => false);
    final repository = LocalWatchHistoryRepository(
      preferences,
      accountScope: testAccountStorageScope(),
    );
    final entry = WatchHistoryEntry(
      videoId: 'e:video-1',
      title: 'A relay-side banger',
      creatorName: 'Nora Relay',
      watchedAt: DateTime.utc(2026, 3, 12, 10),
    );

    await expectLater(
      repository.record(entry),
      throwsA(
        isA<AppFailure>().having(
          (failure) => failure.message,
          'message',
          'Could not save watch history.',
        ),
      ),
    );
  });
}
