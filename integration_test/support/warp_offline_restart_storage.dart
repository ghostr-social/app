import 'dart:io';

import 'package:path_provider/path_provider.dart';

import 'warp_offline_restart_manifest.dart';

final class WarpOfflineRestartStorage {
  const WarpOfflineRestartStorage._(this.root);

  static Future<WarpOfflineRestartStorage> reset() async {
    final storage = WarpOfflineRestartStorage._(await _root());
    await storage.delete();
    await storage.cache.create(recursive: true);
    return storage;
  }

  static Future<WarpOfflineRestartStorage> open() async {
    final storage = WarpOfflineRestartStorage._(await _root());
    if (!await storage.manifestFile.exists()) {
      throw StateError('WARP offline seed manifest is missing.');
    }
    return storage;
  }

  final Directory root;

  Directory get cache => Directory('${root.path}/runtime');
  File get manifestFile => File('${root.path}/seed.json');
  File get eventSnapshotFile =>
      File('${cache.path}/native_video_inventory/nostr-event-cache-v1.json');

  Future<void> write(WarpOfflineRestartManifest manifest) {
    return manifestFile.writeAsString(manifest.encode(), flush: true);
  }

  Future<WarpOfflineRestartManifest> read() async {
    return WarpOfflineRestartManifest.decode(await manifestFile.readAsString());
  }

  Future<void> delete() async {
    if (await root.exists()) await root.delete(recursive: true);
  }

  static Future<Directory> _root() async {
    final support = await getApplicationSupportDirectory();
    return Directory(
      '${support.path}/ghostr-warp-offline-restart',
    );
  }
}
