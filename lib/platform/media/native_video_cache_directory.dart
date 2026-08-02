import 'dart:io';

import 'package:ghostr/core/errors/boundary_failure.dart';

class NativeVideoCacheDirectory {
  const NativeVideoCacheDirectory(this.directory);

  final Directory directory;

  Future<void> initialize() async {
    try {
      if (await directory.exists()) await directory.delete(recursive: true);
      await directory.create(recursive: true);
    } on Object catch (error, stackTrace) {
      throw translatedBoundaryFailure(
        source: 'ghostr.media.native-cache',
        message: 'The native video cache could not be prepared.',
        error: error,
        stackTrace: stackTrace,
      );
    }
  }
}
