import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database_io.dart';

void main() {
  test('opening a corrupt ledger fails instead of deleting it', () async {
    final directory = await Directory.systemTemp.createTemp(
      'ghostr-watch-history-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}history.db';
    await File(path).writeAsString('this is not a sembast database');

    await expectLater(
      openWatchHistoryDatabaseFile(path),
      throwsA(isA<Object>()),
    );
    expect(await File(path).readAsString(), contains('not a sembast'));
  });
}
