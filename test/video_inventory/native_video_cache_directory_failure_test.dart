import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:ghostr/core/errors/app_failure.dart';
import 'package:ghostr/platform/media/native_video_cache_directory.dart';

void main() {
  test('translates a native cache directory creation failure', () async {
    final root = await Directory.systemTemp.createTemp('ghostr-native-');
    addTearDown(() => root.delete(recursive: true));
    final file = File('${root.path}/occupied')..writeAsStringSync('file');
    final cache = NativeVideoCacheDirectory(Directory(file.path));

    final result = cache.initialize();

    await expectLater(result, throwsA(isA<AppFailure>()));
  });
}
