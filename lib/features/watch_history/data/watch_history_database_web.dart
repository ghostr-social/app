import 'package:sembast_web/sembast_web.dart';

Future<Database> openPlatformWatchHistoryDatabase() {
  return databaseFactoryWeb.openDatabase(
    'ghostr_watch_history',
    mode: DatabaseMode.create,
  );
}
