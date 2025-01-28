import 'dart:async';

import 'package:stacked/stacked.dart';
import '../data/video.dart' as video_source;


class FeedViewModel extends BaseViewModel {
  List<video_source.Video> videos = [];
  int prevVideo = 0;
  int actualScreen = 0;


  FeedViewModel() {
    init();
  }

  Future<void> init() async {
    videos.addAll(await video_source.getVideos());
    notifyListeners();
  }


  Future<void> changeVideo(int index) async {
    print('Changing video to $index');

    if (videos.isEmpty) return;
    if (index >= videos.length) return;

    if (videos[index].controller == null) {
      await videos[index].loadController();
    }
    videos[index].controller?.play();

    if (prevVideo != index && videos[prevVideo].controller != null) {
      videos[prevVideo].controller?.pause();
    }

    // Preload neighbors
    final minRange = (index - 1).clamp(0, videos.length - 1);
    final maxRange = (index + 1).clamp(0, videos.length - 1);
    for (int i = 0; i < videos.length; i++) {
      if (i >= minRange && i <= maxRange) {
        if (videos[i].controller == null) {
          videos[i].loadController().catchError((e) {
            print('Error preloading video $i: $e');
          });
        }
      } else {
        // Dispose outside the cache window
        videos[i].controller?.dispose();
        videos[i].controller = null;
      }
    }

    videos.addAll(await video_source.getVideos());
    prevVideo = index;
    notifyListeners();
  }

  Future<void> loadVideo(int index) async {
    if (index < videos.length) {
      try {
        await videos[index].loadController();
        videos[index].controller?.play();
        notifyListeners();
      } catch (e) {
        print('Error loading video at index $index: $e');
      }
    }
  }

  video_source.UserData? currentUserData() {
    if (videos.isEmpty) {
      print('No videos');
      return null;
    }
    try {
      return videos[prevVideo + 1].user;
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




