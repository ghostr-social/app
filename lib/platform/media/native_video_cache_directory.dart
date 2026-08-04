import 'dart:io';

import 'package:ghostr/core/errors/boundary_failure.dart';

class NativeVideoCacheDirectory {
  const NativeVideoCacheDirectory(this.directory);

  final Directory directory;

  /// Ensures the directory exists without disturbing what it holds: the
  /// engine reloads the partial-range store and its host statistics from
  /// here on start, and Rust sweeps the legacy whole-file artifacts.
  Future<void> initialize() async {
    try {
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
