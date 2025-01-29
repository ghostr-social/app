// lib/screens/feed_viewmodel.dart

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

  // If you want to remove a video that is corrupted or cannot load:
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

  // Keep multiple controllers in a pool keyed by the index of the video.
  final Map<int, VideoPlayerController> _controllerPool = {};

  // How many "previous" or "future" videos to keep in memory & preload
  final int nPrevious = 2;
  final int nAhead = 3;

  int currentVideoIndex = 0;
  int actualScreen = 0;

  // The currently "active" video controller
  VideoPlayerController? _activeController;
  VideoPlayerController? get player => _activeController;

  bool _initialFetchDone = false;

  // Used to check concurrency (if multiple changeVideo calls come in quickly).
  int _lastChangeRequestId = 0;

  FeedViewModel() {
    init();
  }

  /// Initial load of videos into the [videoBank].
  Future<void> init() async {
    final initial = await video_source.getVideos();
    videoBank.addAll(initial);
    _initialFetchDone = true;
    notifyListeners();
  }

  /// Called by the UI after your PageView changes to index [newIndex].
  /// Manages preloading, disposing out-of-range videos, and starting the new active video.
  Future<void> changeVideo(int newIndex) async {
    if (newIndex < 0 || newIndex >= videoBank.length) return;

    // Each time changeVideo is called, we increment a request counter.
    final requestId = ++_lastChangeRequestId;

    currentVideoIndex = newIndex;

    // If we are near the end, fetch more videos in the background
    final threshold = videoBank.length - nAhead;
    if (_initialFetchDone && newIndex >= threshold) {
      _fetchMoreVideos();
    }

    // Dispose controllers that are far away from [newIndex].
    _disposeOutOfRange(newIndex);

    // Preload in the background so as not to block the UI.
    // If you want to ensure the *next* video is definitely preloaded, you can
    // await *only* that next video, and let the others be unawaited.
    unawaited(_preloadControllersInRange(newIndex));

    // The new "active" controller
    final candidateController = _controllerPool[newIndex];

    // If another changeVideo call happens while we wait, we should stop further processing here.
    if (requestId != _lastChangeRequestId) return;

    // If we have a preloaded controller, use it; if not, create it *synchronously* now
    // (the user is actually looking at this video).
    if (candidateController == null) {
      await _createAndInitController(newIndex, timeoutSeconds: 5);
    }

    _activeController = _controllerPool[newIndex];

    // It's possible that controller creation or init failed => controller is null
    if (_activeController == null) {
      // The video was dropped due to error
      notifyListeners();
      return;
    }

    // By the time we get here, we want to ensure the active controller is at least initialized.
    if (!_activeController!.value.isInitialized) {
      try {
        await _activeController!.initialize();
      } catch (e) {
        // If there's an error, remove that video
        _removeVideoAtIndex(newIndex);
        notifyListeners();
        return;
      }
    }

    // Play the new video, pause others in the pool
    _playCurrentAndPauseOthers(newIndex);
    notifyListeners();
  }

  /// Fetch more videos from the server and add to the [videoBank].
  /// This runs in background to avoid blocking the user’s scrolling.
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

  /// Dispose controllers that are out of the range [currentIndex - nPrevious, currentIndex + nAhead].
  void _disposeOutOfRange(int currentIndex) {
    final minIndex = (currentIndex - nPrevious).clamp(0, videoBank.length - 1);
    final maxIndex = (currentIndex + nAhead).clamp(0, videoBank.length - 1);

    final keysToRemove = <int>[];
    for (final k in _controllerPool.keys) {
      if (k < minIndex || k > maxIndex) {
        keysToRemove.add(k);
      }
    }
    for (final k in keysToRemove) {
      _controllerPool[k]?.dispose();
      _controllerPool.remove(k);
    }
  }

  /// Preload controllers (in parallel) for [currentIndex - nPrevious .. currentIndex + nAhead].
  Future<void> _preloadControllersInRange(int currentIndex) async {
    final minIndex = (currentIndex - nPrevious).clamp(0, videoBank.length - 1);
    final maxIndex = (currentIndex + nAhead).clamp(0, videoBank.length - 1);

    // Collect tasks for any missing controllers
    final tasks = <Future<void>>[];
    for (int i = minIndex; i <= maxIndex; i++) {
      if (!_controllerPool.containsKey(i)) {
        tasks.add(_createAndInitController(i, timeoutSeconds: 5));
      }
    }

    // You can decide to wait for all or none.
    // Here, we'll wait for them so that the next swipe is definitely smooth.
    if (tasks.isNotEmpty) {
      await Future.wait(tasks);
    }
  }

  /// Creates a [VideoPlayerController], tries to initialize within [timeoutSeconds].
  /// If it fails or times out, we drop that video from the feed.
  Future<void> _createAndInitController(int index, {int timeoutSeconds = 5}) async {
    if (index < 0 || index >= videoBank.length) return;
    if (_controllerPool.containsKey(index)) return; // Already created

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
      // If initialization completes, store in pool only if still valid.
      // It's possible user scrolled far away and we already disposed stuff:
      if (!_controllerPool.containsKey(index)) {
        _controllerPool[index] = controller;
      } else {
        // If, for some reason, we already replaced it, dispose this new one.
        controller.dispose();
      }
    } catch (e) {
      // Possibly timed out or got an error -> remove the video from the feed
      print("Dropping video ${vid.id} due to error/timeout: $e");
      controller.dispose();
      // Double-check if video is still in the bank at that index
      // to avoid removing the wrong one if the bank has changed
      if (index < videoBank.length && videoBank[index].id == vid.id) {
        _removeVideoAtIndex(index);
      }
    }
  }

  /// Plays the [activeIdx] controller (if any), and pauses all others in the pool.
  void _playCurrentAndPauseOthers(int activeIdx) {
    _controllerPool.forEach((idx, ctl) {
      if (idx == activeIdx) {
        if (ctl.value.isInitialized) {
          ctl.play();
        }
      } else {
        if (ctl.value.isInitialized && ctl.value.isPlaying) {
          ctl.pause();
        }
      }
    });
  }

  /// Removes the video from the feed and shifts indices as needed.
  void _removeVideoAtIndex(int index) {
    if (index < 0 || index >= videoBank.length) return;
    final removedID = videoBank[index].id;
    videoBank.removeVideo(removedID);

    // Dispose that controller if it exists
    final existingController = _controllerPool.remove(index);
    existingController?.dispose();

    // Re-map any controllers after this index because the list shrinks by 1
    final oldPool = Map<int, VideoPlayerController>.from(_controllerPool);
    _controllerPool.clear();

    // For each old key, figure out the new key:
    // Everything after `index` shifts left by 1
    oldPool.forEach((oldIdx, oldController) {
      if (oldIdx < index) {
        _controllerPool[oldIdx] = oldController;
      } else if (oldIdx > index) {
        _controllerPool[oldIdx - 1] = oldController;
      }
    });

    // If currentVideoIndex was after this index, shift it too
    if (currentVideoIndex > index) {
      currentVideoIndex--;
    }
  }

  /// Get the user data for the currently playing video
  video_source.UserData? currentUserData() {
    if (videoBank.isEmpty) return null;
    if (currentVideoIndex < 0 || currentVideoIndex >= videoBank.length) {
      return null;
    }
    return videoBank[currentVideoIndex].user;
  }

  /// Called from bottom bar to change which screen is displayed
  void setActualScreen(int index) {
    actualScreen = index;
    notifyListeners();
  }

  @override
  void dispose() {
    for (final ctl in _controllerPool.values) {
      ctl.dispose();
    }
    _controllerPool.clear();
    super.dispose();
  }
}
