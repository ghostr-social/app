import 'dart:async';
import 'dart:collection';

import 'package:stacked/stacked.dart';
import '../data/video.dart' as video_source;


class VideoBank extends IterableBase<video_source.Video> {
  final List<video_source.Video> _videos = [];
  final HashSet<String> _knownVideos = HashSet<String>();

  void addAll(List<video_source.Video> newVideos) {
    for (var video in newVideos) {
      if (!_knownVideos.contains(video.id)) {
        _videos.add(video);
        _knownVideos.add(video.id);
      }
    }
  }

  @override
  Iterator<video_source.Video> get iterator => _videos.iterator;

  // Optional helper getters
  @override
  int get length => _videos.length;
  @override
  bool get isEmpty => _videos.isEmpty;
  video_source.Video operator [](int index) => _videos[index];
}



class FeedViewModel extends BaseViewModel {
  VideoBank videoBank = VideoBank();
  int prevVideo = 0;
  int actualScreen = 0;


  FeedViewModel() {
    init();
  }

  Future<void> init() async {
    videoBank.addAll(await video_source.getVideos());
    notifyListeners();
  }


  Future<void> changeVideo(int index) async {
    print('Changing video to $index');

    if (videoBank.isEmpty) return;
    if (index >= videoBank.length) return;

    if (videoBank[index].controller == null) {
      await videoBank[index].loadController();
    }
    videoBank[index].controller?.play();

    if (prevVideo != index && videoBank[prevVideo].controller != null) {
      videoBank[prevVideo].controller?.pause();
    }

    // Preload neighbors
    final minRange = (index - 1).clamp(0, videoBank.length - 1);
    final maxRange = (index + 1).clamp(0, videoBank.length - 1);
    for (int i = 0; i < videoBank.length; i++) {
      if (i >= minRange && i <= maxRange) {
        if (videoBank[i].controller == null) {
          videoBank[i].loadController().catchError((e) {
            print('Error preloading video $i: $e');
          });
        }
      } else {
        // Dispose outside the cache window
        videoBank[i].controller?.dispose();
        videoBank[i].controller = null;
      }
    }

    videoBank.addAll(await video_source.getVideos());
    prevVideo = index;
    notifyListeners();
  }

  Future<void> loadVideo(int index) async {
    if (index < videoBank.length) {
      try {
        await videoBank[index].loadController();
        videoBank[index].controller?.play();
        notifyListeners();
      } catch (e) {
        print('Error loading video at index $index: $e');
      }
    }
  }

  video_source.UserData? currentUserData() {
    if (videoBank.isEmpty) {
      print('No videos');
      return null;
    }
    try {
      return videoBank[prevVideo + 1].user;
    } catch (e) {
      print('Error getting userData: $e');
    }
    return null;
  }

  void setActualScreen(int index) {
    actualScreen = index;
    notifyListeners();
  }

}




