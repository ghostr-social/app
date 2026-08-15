import 'package:flutter_test/flutter_test.dart';
import 'package:sembast/sembast_memory.dart';

var _databaseNumber = 0;

Future<Database> openTestWatchHistoryDatabase() async {
  final database = await databaseFactoryMemory.openDatabase(
    'watch-history-test-${_databaseNumber++}',
    mode: DatabaseMode.create,
  );
  addTearDown(database.close);
  return database;
}
