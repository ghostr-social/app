// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.regex.Pattern;

final class WarpPlaybackAttemptHeaders {
  static final String HEADER_NAME = "X-Ghostr-Playback-Attempt";
  private static final Pattern TOKEN_PATTERN =
      Pattern.compile("[A-Za-z0-9_-]{21}[AQgw]");

  static final class Result {
    @NonNull private final Map<String, String> requestHeaders;
    @Nullable private final String attemptToken;

    Result(@NonNull Map<String, String> requestHeaders, @Nullable String attemptToken) {
      this.requestHeaders = Collections.unmodifiableMap(requestHeaders);
      this.attemptToken = attemptToken;
    }

    @NonNull
    Map<String, String> getRequestHeaders() {
      return requestHeaders;
    }

    @Nullable
    String getAttemptToken() {
      return attemptToken;
    }
  }

  @NonNull
  static Result extract(@NonNull Map<String, String> headers) {
    Map<String, String> requestHeaders = new LinkedHashMap<>();
    String attemptToken = null;
    boolean found = false;
    for (Map.Entry<String, String> header : headers.entrySet()) {
      if (!HEADER_NAME.equalsIgnoreCase(header.getKey())) {
        requestHeaders.put(header.getKey(), header.getValue());
        continue;
      }
      if (found) {
        throw new IllegalArgumentException("Duplicate " + HEADER_NAME + " header");
      }
      found = true;
      attemptToken = requireValidToken(header.getValue());
    }
    return new Result(requestHeaders, attemptToken);
  }

  @NonNull
  private static String requireValidToken(@Nullable String token) {
    if (token == null || !TOKEN_PATTERN.matcher(token).matches()) {
      throw new IllegalArgumentException("Invalid " + HEADER_NAME + " token");
    }
    return token;
  }

  private WarpPlaybackAttemptHeaders() {}
}
