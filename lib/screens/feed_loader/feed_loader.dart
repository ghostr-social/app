import 'package:flutter/material.dart';

import '../../data/video.dart';
import '../feed_screen/feed_screen.dart';

class FeedLoader extends StatefulWidget {
  const FeedLoader({super.key});

  @override
  _FeedLoaderState createState() => _FeedLoaderState();
}

class _FeedLoaderState extends State<FeedLoader> {
  late Future<List<Video>> _feedDataFuture;

  @override
  void initState() {
    super.initState();
    _feedDataFuture = _retrieveInitialData();
  }

  Future<List<Video>> _retrieveInitialData() async {
    List<Video> videos = [];

    while (videos.length < 10) {
      videos.addAll(await getVideos());
      if (videos.length < 10) {
        await Future.delayed(const Duration(seconds: 2));
      }
    }

    return videos;
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<Video>>(
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
          print('Loaded ${initialVideos.length} videos');
          return FeedScreen(initialVideos: initialVideos,);
        }
      },
    );
  }
}