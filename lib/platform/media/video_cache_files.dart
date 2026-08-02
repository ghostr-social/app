import 'dart:io';

class VideoCacheRequest {
  const VideoCacheRequest({
    required this.directory,
    required this.sources,
    required this.completed,
    required this.partial,
  });

  final Directory directory;
  final List<Uri> sources;
  final File completed;
  final File partial;
}

class VideoCacheFile {
  const VideoCacheFile(this.file, this.bytes, this.modified);

  final File file;
  final int bytes;
  final DateTime modified;
}
