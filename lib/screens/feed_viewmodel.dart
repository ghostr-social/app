import 'package:flutter/services.dart';
import 'package:stacked/stacked.dart';
import 'package:video_player/video_player.dart';

import '../data/video.dart';
import '../data/videos_repository.dart';


class FeedViewModel extends BaseViewModel {
  VideoPlayerController? controller;
  VideosAPI? videoSource;

  int prevVideo = 0;

  int actualScreen = 0;

  FeedViewModel() {
    videoSource = VideosAPI();
  }

  changeVideo(int index) async {
    final videos = videoSource!.listVideos;
    if (videos.isEmpty) return;

    // 1. Load & play the new index if needed
    if (videos[index].controller == null) {
      await videos[index].loadController();
    }
    videos[index].controller?.play();

    // 2. Pause the old video
    if (prevVideo != index && videos[prevVideo].controller != null) {
      videos[prevVideo].controller?.pause();
    }

    // 3. Preload a small range [index-3 .. index+4]
    final int minRange = (index - 3).clamp(0, videos.length - 1);
    final int maxRange = (index + 4).clamp(0, videos.length - 1);

    for (int i = 0; i < videos.length; i++) {
      if (i >= minRange && i <= maxRange) {
        // If in our preload range, load it if not already loaded
        if (videos[i].controller == null) {
          // Fire & forget—so we don’t block the user
          videos[i].loadController().catchError((e) {
            print('Error preloading video at index $i: $e');
          });
        }
      } else {
        // 4. Dispose controllers outside our cache window
        if (videos[i].controller != null) {
          videos[i].controller!.dispose();
          videos[i].controller = null;
        }
      }
    }

    prevVideo = index;
    notifyListeners();
    print('Now playing video index: $index');
  }

  UserData? currentUserData() {
    try {
      return videoSource!.listVideos[prevVideo+1].user;
    } catch(e){
      print('Error getting userData: $e');
    }
    return null;
  }
  void loadVideo(int index) async {
    if (videoSource!.listVideos.length > index) {
      try {
        await videoSource!.listVideos[index].loadController();
        videoSource!.listVideos[index].controller?.play();
        notifyListeners();
      } catch (e) {
        print('Error loading video at index $index: $e');
      }
    }
  }

  void setActualScreen(index) {
    actualScreen = index;
    if (index == 0) {
      SystemChrome.setSystemUIOverlayStyle(SystemUiOverlayStyle.light);
    } else {
      SystemChrome.setSystemUIOverlayStyle(SystemUiOverlayStyle.dark);
    }
    notifyListeners();
  }
}
