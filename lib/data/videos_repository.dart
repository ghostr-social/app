import 'dart:async';
import 'package:hookstr/data/video.dart';
import 'package:nostr_sdk/nostr_sdk.dart';

class VideoVariant {
  final String? resolution;
  final String? url;
  final String? hash;
  final String? mimeType;
  final List<String> images;
  final List<String> fallbacks;
  final String? service;

  const VideoVariant({
    this.resolution,
    this.url,
    this.hash,
    this.mimeType,
    this.images = const [],
    this.fallbacks = const [],
    this.service,
  });
}



class VideosAPI {
  // Store fetched videos in memory
  List<Video> listVideos = <Video>[];

  VideosAPI() {
    load();
  }

  Future<void> load() async {
    listVideos = await getVideoList();
  }

  Future<List<Video>> getVideoList() async {
    final client = Client.builder().build();

    await client.addRelay(url: 'wss://relay.damus.io');
    await client.addRelay(url: 'wss://relay.snort.social');
    await client.connect();

    try {
      final filter = Filter()
          .kind(kind: 34235)
          .kind(kind: 34236);

      final events = await client.fetchEvents(
        filters: [filter],
        timeout: const Duration(seconds: 10),
      );

      final videos = events
          .map((event) => _parseEventAsVideo(event))
          .whereType<Video>()
          .toList();

      return videos;
    } catch (e, st) {
      print('Error fetching video events: $e $st');
      return [];
    } finally {
      await client.disconnect();
    }
  }

  Video? _parseEventAsVideo(Event event) {
    try {
      final tags = event.tags();
      final videoVariants = <VideoVariant>[];

      for (final t in tags) {
         var vec = t.asVec();
        if (vec.isNotEmpty && vec[0] == 'imeta') {
          final fields = <String, List<String>>{};

          for (int i = 1; i < vec.length; i++) {
            final parts = vec[i].split(' ');
            if (parts.isEmpty) continue;

            final key = parts[0].trim();
            final value = parts.sublist(1).join(' ').trim();
            fields.putIfAbsent(key, () => []);
            fields[key]!.add(value);
          }

          final dim = fields['dim']?.first;
          final url = fields['url']?.first;
          final hash = fields['x']?.first;
          final mimeType = fields['m']?.first;
          final service = fields['service']?.first;
          final images = fields['image'] ?? [];
          final fallbacks = fields['fallback'] ?? [];

          final variant = VideoVariant(
            resolution: dim,
            url: url,
            hash: hash,
            mimeType: mimeType,
            images: images,
            fallbacks: fallbacks,
            service: service,
          );

          videoVariants.add(variant);
        }
      }


      if (videoVariants.isEmpty) {
        return null;
      }

      var video = videoVariants[0];

      if(video.url == null || video.url == ""){
        return null;
      }

      print(video.url);
      return Video(
        id: video.hash!,
        user: event.author().toBech32(),
        userPic: "",
        videoTitle: "NIP–71 Video",
        songName: "Unknown",
        comments: "",
        likes: '',
        url: video.url!,
      );
    } catch (err) {
      print('Error parsing video event: $err');
      return null;
    }
  }
}