import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/features/watch_history/data/watch_history_database_io.dart';
import 'package:sembast/sembast.dart';

void main() {
  test('native watch history uses a transactional SQLite journal', () async {
    final directory = await Directory.systemTemp.createTemp(
      'ghostr-watch-sqlite-',
    );
    addTearDown(() => directory.delete(recursive: true));
    final path = '${directory.path}${Platform.pathSeparator}history.db';
    final database = await openWatchHistoryDatabaseFile(path);
    await stringMapStoreFactory.store('ledger').record('seen').put(
      database,
      <String, Object?>{'value': true},
    );
    await database.close();

    final header = await File(path)
        .openRead(0, 16)
        .fold<List<int>>(<int>[], (bytes, chunk) => bytes..addAll(chunk));
    expect(ascii.decode(header), 'SQLite format 3\u0000');
  });
}
