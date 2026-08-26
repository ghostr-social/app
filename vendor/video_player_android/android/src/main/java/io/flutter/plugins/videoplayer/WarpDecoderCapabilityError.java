// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import androidx.annotation.NonNull;
import androidx.media3.common.C;
import androidx.media3.common.Format;
import androidx.media3.common.PlaybackException;
import androidx.media3.exoplayer.ExoPlaybackException;
import java.util.HashMap;
import java.util.Map;

final class WarpDecoderCapabilityError {
  static final String ERROR_CODE = "VideoDecoderUnsupported";

  static boolean isDefinitive(@NonNull PlaybackException error) {
    if (!(error instanceof ExoPlaybackException exo)
        || exo.type != ExoPlaybackException.TYPE_RENDERER
        || !isVideo(exo.rendererFormat)
        || exo.rendererFormatSupport == C.FORMAT_UNSUPPORTED_DRM) {
      return false;
    }
    int support = exo.rendererFormatSupport;
    return error.errorCode == PlaybackException.ERROR_CODE_DECODING_FORMAT_EXCEEDS_CAPABILITIES
        || error.errorCode == PlaybackException.ERROR_CODE_DECODING_FORMAT_UNSUPPORTED
        || support == C.FORMAT_UNSUPPORTED_TYPE
        || support == C.FORMAT_UNSUPPORTED_SUBTYPE
        || support == C.FORMAT_EXCEEDS_CAPABILITIES;
  }

  private static boolean isVideo(Format format) {
    return format != null
        && format.sampleMimeType != null
        && format.sampleMimeType.startsWith("video/");
  }

  @NonNull
  static String message(@NonNull PlaybackException error) {
    return "[VideoDecoderUnsupported] Video decoder cannot play selected format: " + format(error);
  }

  @NonNull
  static Map<String, Object> details(@NonNull PlaybackException error) {
    Map<String, Object> details = new HashMap<>();
    details.put("errorCode", error.errorCode);
    details.put("errorCodeName", error.getErrorCodeName());
    if (error instanceof ExoPlaybackException exo) {
      details.put("formatSupport", exo.rendererFormatSupport);
      addFormat(details, exo.rendererFormat);
    }
    return details;
  }

  @NonNull
  private static String format(@NonNull PlaybackException error) {
    if (error instanceof ExoPlaybackException exo && exo.rendererFormat != null) {
      return exo.rendererFormat.toString();
    }
    return error.getErrorCodeName();
  }

  private static void addFormat(Map<String, Object> details, Format format) {
    if (format == null) return;
    if (format.sampleMimeType != null) details.put("mimeType", format.sampleMimeType);
    if (format.codecs != null) details.put("codecs", format.codecs);
    if (format.width != Format.NO_VALUE) details.put("width", format.width);
    if (format.height != Format.NO_VALUE) details.put("height", format.height);
  }

  private WarpDecoderCapabilityError() {}
}
