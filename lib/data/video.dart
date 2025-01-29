import 'dart:io';

import 'package:flutter/foundation.dart';
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

  static UserData fromMap(Map<String, dynamic> map) {
    return UserData(
      npub: map['npub'],
      name: map['name'],
      profilePicture: map['profilePicture'],
    );
  }

  Map<String, dynamic> toMap() {
    return {
      'npub': npub,
      'name': name,
      'profilePicture': profilePicture,
    };
  }
}

class Video {
  String id;
  UserData user;
  String videoTitle;
  String songName;
  String likes;
  String comments;
  String url;
  String? localPath;


  Video(
      {required this.id,
      required this.user,
      required this.videoTitle,
      required this.songName,
      required this.likes,
      required this.comments,
      required this.url,
      this.localPath});

 static Video fromMap(Map<String, dynamic> map) {
    return Video(
      id: map['id'],
      user: UserData.fromMap(map['user']),
      videoTitle: map['videoTitle'],
      songName: map['songName'],
      likes: map['likes'],
      comments: map['comments'],
      url: map['url'],
      localPath: map['localPath'] ?? null,
    );
  }

  Map<String, dynamic> toMap() {
    return {
      'id': id,
      'user': user.toMap(),
      'videoTitle': videoTitle,
      'songName': songName,
      'likes': likes,
      'comments': comments,
      'url': url,
      'localPath': localPath ?? null,
    };
  }

}


Future<List<Video>> getVideos() async {
  if (kDebugMode) print("[getVideos] Getting videos");
  var videos = await ffiGetDiscoveredVideos();
  if (kDebugMode) print("[ffiGetDiscoveredVideos] Got ${videos.length} videos");
  return videos.map((e) => Video(
    id: e.id,
    user: UserData(name: "Unknown", profilePicture: null),
    videoTitle: e.title ?? "Unknown",
    songName: "Unknown",
    likes: "0",
    comments: "0",
    url: e.url,
    localPath: e.localPath,
  )).toList();
}

