import 'package:flutter/material.dart';
import 'package:ghostr/configs/base.dart';
import 'package:ghostr/src/rust/video/video.dart';
import 'package:video_player/video_player.dart';

class UserData {
  final String? npub;
  final String? name;
  final String? profilePicture;
  const UserData({
    this.npub,
    this.name,
    this.profilePicture,
  });
}

class Video {
  String id;
  UserData user;
  String videoTitle;
  String songName;
  String likes;
  String comments;
  String url;

  VideoPlayerController? controller;

  Video(
      {required this.id,
      required this.user,
      required this.videoTitle,
      required this.songName,
      required this.likes,
      required this.comments,
      required this.url});

  Future<Null> loadController(int index) async {
    debugPrint("loading $url");
    var uri = Uri.parse("$baseLoopbackUrl/video.mp4?=$index");
    debugPrint("loading $uri");

    controller = VideoPlayerController.networkUrl(uri);
    await controller?.initialize();
    controller?.setLooping(true);
  }
}


Future<List<Video>> getVideos() async {
  var videos = await ffiGetDiscoveredVideos();
  return videos.map((e) => Video(
    id: e.id,
    user: UserData(name: "Unknown", profilePicture: null),
    videoTitle: e.title ?? "Unknown",
    songName: "Unknown",
    likes: "0",
    comments: "0",
    url: e.url,
  )).toList();
}