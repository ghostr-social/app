import 'package:sembast/sembast.dart';

import 'watch_history_database_io.dart'
    if (dart.library.js_interop) 'watch_history_database_web.dart'
    as platform;

typedef WatchHistoryDatabaseLoader = Future<Database> Function();

Future<Database> openWatchHistoryDatabase() {
  return platform.openPlatformWatchHistoryDatabase();
}
