import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import 'package:sembast/sembast.dart';
import 'package:sembast_sqflite/sembast_sqflite.dart';
import 'package:sqflite/sqflite.dart' as sqflite;
import 'package:sqflite_common_ffi/sqflite_ffi.dart' as ffi;

Future<Database> openPlatformWatchHistoryDatabase() async {
  final directory = await getApplicationSupportDirectory();
  final path =
      '${directory.path}${Platform.pathSeparator}ghostr_watch_history.sqlite';
  if (_usesFlutterSqlitePlugin(defaultTargetPlatform)) {
    return getDatabaseFactorySqflite(
      sqflite.databaseFactory,
    ).openDatabase(path, mode: DatabaseMode.create);
  }
  return openWatchHistoryDatabaseFile(path);
}

Future<Database> openWatchHistoryDatabaseFile(String path) {
  ffi.sqfliteFfiInit();
  return getDatabaseFactorySqflite(
    ffi.databaseFactoryFfi,
  ).openDatabase(path, mode: DatabaseMode.create);
}

bool _usesFlutterSqlitePlugin(TargetPlatform platform) {
  return platform == TargetPlatform.android ||
      platform == TargetPlatform.iOS ||
      platform == TargetPlatform.macOS;
}
