import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../data/video.dart';
import '../../src/rust/video/video.dart';
import '../feed_screen/feed_screen.dart';


const minVideosToStartFeed = 3;

class FeedLoader extends StatefulWidget {
  const FeedLoader({super.key});

  @override
  _FeedLoaderState createState() => _FeedLoaderState();
}

class _FeedLoaderState extends State<FeedLoader> {
  late Future<List<FfiVideoDownload>> _feedDataFuture;

  @override
  void initState() {
    super.initState();
    _feedDataFuture = _retrieveInitialData();
  }

  Future<List<FfiVideoDownload>> _retrieveInitialData() async {
    List<FfiVideoDownload> videos = [];

    while (videos.length < minVideosToStartFeed) {
      videos.addAll(await getVideos());
      if (videos.length < minVideosToStartFeed) {
        await Future.delayed(const Duration(seconds: 2));
      }
    }

    return videos;
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<FfiVideoDownload>>(
      future: _feedDataFuture,
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          // While data is loading, show an animation/placeholder
          return const Center(
            child: CircularProgressIndicator(), // or any custom animation
          );
        } else if (snapshot.hasError) {
          // Optionally handle errors here
          return Center(
            child: Text('Error: ${snapshot.error}'),
          );
        } else {
          var initialVideos =  snapshot.data!;
          if (kDebugMode) print('Loaded ${initialVideos.length} videos');
          return FeedScreen(initialVideos: initialVideos,);
        }
      },
    );
  }
}