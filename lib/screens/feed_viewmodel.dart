import 'dart:async';
import 'dart:collection';
import 'dart:io';

import 'package:stacked/stacked.dart';
import 'package:video_player/video_player.dart';

import '../configs/base.dart';
import '../data/video.dart' as video_source;

class VideoBank extends Iterable<video_source.Video> {
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

  void removeVideo(String videoId) {
    _videos.removeWhere((v) => v.id == videoId);
    _knownVideos.remove(videoId);
  }

  @override
  Iterator<video_source.Video> get iterator => _videos.iterator;

  @override
  int get length => _videos.length;

  @override
  bool get isEmpty => _videos.isEmpty;

  video_source.Video operator [](int index) => _videos[index];
}

class FeedViewModel extends BaseViewModel {
  final VideoBank videoBank = VideoBank();

  // Pool of controllers keyed by the index of the video in videoBank.
  final Map<int, VideoPlayerController> controllerPool = {};

  // How many to keep preloaded behind and ahead.
  final int nPrevious = 2;
  final int nAhead = 3;

  int currentVideoIndex = 0;
  int actualScreen = 0;

  // The currently "active" video controller
  VideoPlayerController? _activeController;
  VideoPlayerController? get player => _activeController;

  bool _initialFetchDone = false;
  int _lastChangeRequestId = 0;

  FeedViewModel() {
    init();
  }

  Future<void> init() async {
    final initial = await video_source.getVideos();
    videoBank.addAll(initial);
    _initialFetchDone = true;
    notifyListeners();
  }

  /// Called by the UI after PageView changes to index [newIndex].
  Future<void> changeVideo(int newIndex) async {
    if (newIndex < 0 || newIndex >= videoBank.length) return;

    // 1) Immediately pause the old video so it doesn't keep playing.
    if (_activeController != null && _activeController!.value.isPlaying) {
      _activeController!.pause();
    }

    // Each call increments a request counter, used to cancel old operations.
    final requestId = ++_lastChangeRequestId;
    currentVideoIndex = newIndex;

    // Attempt to fetch more videos in the background if near the end.
    final threshold = videoBank.length - nAhead;
    if (_initialFetchDone && newIndex >= threshold) {
      unawaited(_fetchMoreVideos());
    }

    // Dispose controllers that are far from newIndex.
    _disposeOutOfRange(newIndex);

    // Preload others around newIndex.
    unawaited(_preloadControllersInRange(newIndex));

    // 2) If there's already a preloaded controller for the newIndex, use it.
    var candidateController = controllerPool[newIndex];
    if (candidateController == null) {
      await _createAndInitController(newIndex, timeoutSeconds: 5);
      candidateController = controllerPool[newIndex];
    }

    // Check if a newer changeVideo request has superseded this one.
    if (requestId != _lastChangeRequestId) return;

    _activeController = candidateController;

    if (_activeController == null) {
      // The video might have been dropped due to an error.
      notifyListeners();
      return;
    }

    // Ensure it's initialized before playing.
    if (!_activeController!.value.isInitialized) {
      try {
        await _activeController!.initialize();
      } catch (e) {
        _removeVideoAtIndex(newIndex);
        notifyListeners();
        return;
      }
    }

    // 3) Play the newly active video, and reset all others.
    _playCurrentAndResetOthers(newIndex);

    notifyListeners();
  }

  Future<void> _fetchMoreVideos() async {
    try {
      final more = await video_source.getVideos();
      if (more.isNotEmpty) {
        videoBank.addAll(more);
        notifyListeners();
      }
    } catch (e) {
      print('Error fetching more videos: $e');
    }
  }

  void _disposeOutOfRange(int currentIndex) {
    final minIndex = (currentIndex - nPrevious).clamp(0, videoBank.length - 1);
    final maxIndex = (currentIndex + nAhead).clamp(0, videoBank.length - 1);

    final keysToRemove = <int>[];
    for (final k in controllerPool.keys) {
      if (k < minIndex || k > maxIndex) {
        keysToRemove.add(k);
      }
    }

    for (final k in keysToRemove) {
      controllerPool[k]?.dispose();
      controllerPool.remove(k);
    }
  }

  Future<void> _preloadControllersInRange(int currentIndex) async {
    final minIndex = (currentIndex - nPrevious).clamp(0, videoBank.length - 1);
    final maxIndex = (currentIndex + nAhead).clamp(0, videoBank.length - 1);

    final tasks = <Future<void>>[];
    for (int i = minIndex; i <= maxIndex; i++) {
      if (!controllerPool.containsKey(i)) {
        tasks.add(_createAndInitController(i, timeoutSeconds: 2));
      }
    }
    if (tasks.isNotEmpty) {
      await Future.wait(tasks);
    }
  }

  Future<void> _createAndInitController(int index, {int timeoutSeconds = 1}) async {
    if (index < 0 || index >= videoBank.length) return;
    if (controllerPool.containsKey(index)) return;

    final vid = videoBank[index];
    VideoPlayerController controller;

    if (vid.localPath != null) {
      controller = VideoPlayerController.file(File(vid.localPath!));
    } else {
      final uri = Uri.parse("$baseLoopbackUrl/video.mp4?id=${vid.id}");
      controller = VideoPlayerController.networkUrl(uri);
    }
    controller.setLooping(true);

    try {
      await controller.initialize().timeout(Duration(seconds: timeoutSeconds));
      // If still valid, add to pool. Otherwise dispose.
      if (!controllerPool.containsKey(index)) {
        controllerPool[index] = controller;
      } else {
        controller.dispose();
      }
    } catch (e) {
      print("Dropping video ${vid.id} due to error/timeout: $e");
      controller.dispose();

      // Double-check if it's still the same video in the bank.
      if (index < videoBank.length && videoBank[index].id == vid.id) {
        _removeVideoAtIndex(index);
      }
    }
  }

  /// Pauses and resets every controller except [activeIdx], which is played.
  void _playCurrentAndResetOthers(int activeIdx) {
    controllerPool.forEach((idx, ctl) {
      if (idx == activeIdx) {
        if (ctl.value.isInitialized) {
          ctl.play(); // If you want the active video to start at zero, do: ctl.seekTo(Duration.zero);
        }
      } else {
        if (ctl.value.isInitialized) {
          ctl.pause();
          ctl.seekTo(Duration.zero); // Ensure any inactive video is at the start.
        }
      }
    });
  }

  void _removeVideoAtIndex(int index) {
    if (index < 0 || index >= videoBank.length) return;
    final removedID = videoBank[index].id;
    videoBank.removeVideo(removedID);

    final existingController = controllerPool.remove(index);
    existingController?.dispose();

    // Re-map controllers after removing one video from the list
    final oldPool = Map<int, VideoPlayerController>.from(controllerPool);
    controllerPool.clear();

    oldPool.forEach((oldIdx, oldController) {
      if (oldIdx < index) {
        controllerPool[oldIdx] = oldController;
      } else if (oldIdx > index) {
        controllerPool[oldIdx - 1] = oldController;
      }
    });

    if (currentVideoIndex > index) {
      currentVideoIndex--;
    }
  }

  video_source.UserData? currentUserData() {
    if (videoBank.isEmpty) return null;
    if (currentVideoIndex < 0 || currentVideoIndex >= videoBank.length) {
      return null;
    }
    return videoBank[currentVideoIndex].user;
  }

  void setActualScreen(int index) {
    actualScreen = index;
    notifyListeners();
  }

  @override
  void dispose() {
    for (final ctl in controllerPool.values) {
      ctl.dispose();
    }
    controllerPool.clear();
    super.dispose();
  }
}
