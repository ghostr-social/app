import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/nostr/nostr_event_identity.dart';
import 'package:ghostr/core/storage/account_storage_scope.dart';
import 'package:ghostr/features/activity/data/local_activity_repository.dart';
import 'package:ghostr/features/activity/domain/activity_item.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../support/nostr_test_values.dart';
import '../support/sample_data.dart';

class _Preferences extends Mock implements SharedPreferences {}

void main() {
  test('concurrent activity records retain both items', () async {
    final preferences = _Preferences();
    final firstWrite = Completer<void>();
    final release = Completer<void>();
    String? stored;
    var writes = 0;
    when(() => preferences.getString(any())).thenAnswer((_) => stored);
    when(() => preferences.setString(any(), any())).thenAnswer((call) async {
      writes += 1;
      if (writes == 1) {
        firstWrite.complete();
        await release.future;
      }
      stored = call.positionalArguments[1] as String;
      return true;
    });
    final repository = LocalActivityRepository(
      preferences,
      accountScope: AccountStorageScope(
        () => NostrPublicKeyHex.parse(testViewerPublicKey),
      ),
    );

    final first = repository.record(sampleActivity());
    await firstWrite.future;
    final second = repository.record(_secondActivity());
    await Future<void>.delayed(Duration.zero);
    release.complete();
    await Future.wait(<Future<void>>[first, second]);

    expect((await repository.load()).map((item) => item.id.value), {
      'activity-1',
      'activity-2',
    });
  });
}

ActivityItem _secondActivity() {
  return ActivityItem(
    id: ActivityId.parse('activity-2'),
    type: sampleActivity().type,
    description: sampleActivity().description,
    occurredAt: DateTime(2026, 3, 12, 11),
  );
}
