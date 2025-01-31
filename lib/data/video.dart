import 'package:flutter/foundation.dart';
import 'package:ghostr/src/rust/video/video.dart';


Future<List<FfiVideoDownload>> getVideos() async {
  if (kDebugMode) print("[getVideos] Getting videos");
  var videos = await ffiGetDiscoveredVideos();
  if (kDebugMode) print("[ffiGetDiscoveredVideos] Got ${videos.length} videos");
  return videos;
}

