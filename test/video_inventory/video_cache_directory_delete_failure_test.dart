import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/platform/media/video_cache_directory.dart';

void main() {
  test('a busy cache file cannot block another eligible eviction', () async {
    final directory = await Directory.systemTemp.createTemp('ghostr-cache-');
    addTearDown(() => directory.delete(recursive: true));
    final busy = File('${directory.path}/busy.video');
    final eligible = File('${directory.path}/eligible.video');
    await busy.writeAsBytes(<int>[1, 2, 3, 4]);
    await eligible.writeAsBytes(<int>[5, 6, 7, 8]);
    await busy.setLastModified(DateTime(2000));
    await eligible.setLastModified(DateTime(2001));
    final attempts = <String>[];
    final cache = VideoCacheDirectory(
      4,
      <String>{},
      <String>{},
      deleteFile: (file) async {
        attempts.add(file.path);
        if (file.path == busy.path) {
          throw FileSystemException('File is open', file.path);
        }
        await file.delete();
      },
    );

    await cache.enforceBudget(directory);

    expect(attempts, <String>[busy.path, eligible.path]);
    expect(await busy.exists(), isTrue);
    expect(await eligible.exists(), isFalse);
  });
}
