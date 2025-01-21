// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.7.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These function are ignored because they are on traits that is not defined in current crate (put an empty `#[frb]` on it to unignore): `clone`, `fmt`

/// Start the Axum server and store the AppState in GLOBAL_STATE.
/// Return the bound address as a String.
Future<String> ffiStartServer({String? address}) =>
    RustLib.instance.api.crateVideoVideoFfiStartServer(address: address);

/// Return the discovered videos from the stored AppState.
Future<List<FfiVideoDownload>> ffiGetDiscoveredVideos() =>
    RustLib.instance.api.crateVideoVideoFfiGetDiscoveredVideos();

class FfiVideoDownload {
  final String id;
  final String url;
  final String? title;

  const FfiVideoDownload({
    required this.id,
    required this.url,
    this.title,
  });

  @override
  int get hashCode => id.hashCode ^ url.hashCode ^ title.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is FfiVideoDownload &&
          runtimeType == other.runtimeType &&
          id == other.id &&
          url == other.url &&
          title == other.title;
}
