// Copyright 2013 The Flutter Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

package io.flutter.plugins.videoplayer;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertThrows;

import java.util.LinkedHashMap;
import java.util.Map;
import org.junit.Test;

public final class WarpPlaybackAttemptHeadersTest {
  private static final String TOKEN = "AAAAAAAAAAAAAAAAAAAAAA";

  @Test
  public void extractsOneCanonicalTokenAndPreservesOnlyRequestHeaders() {
    Map<String, String> input = new LinkedHashMap<>();
    input.put("Authorization", "Bearer secret");
    input.put("x-ghostr-playback-attempt", TOKEN);
    input.put("Range-Hint", "preserve");

    WarpPlaybackAttemptHeaders.Result result = WarpPlaybackAttemptHeaders.extract(input);

    assertEquals(TOKEN, result.getAttemptToken());
    assertEquals("Bearer secret", result.getRequestHeaders().get("Authorization"));
    assertEquals("preserve", result.getRequestHeaders().get("Range-Hint"));
    assertFalse(result.getRequestHeaders().containsKey("x-ghostr-playback-attempt"));
  }

  @Test
  public void absentReservedHeaderLeavesOrdinaryHeadersAndNoAttempt() {
    Map<String, String> input = Map.of("Authorization", "ordinary");

    WarpPlaybackAttemptHeaders.Result result = WarpPlaybackAttemptHeaders.extract(input);

    assertNull(result.getAttemptToken());
    assertEquals(input, result.getRequestHeaders());
  }

  @Test
  public void rejectsDuplicateHeaderNamesIgnoringCase() {
    Map<String, String> input = new LinkedHashMap<>();
    input.put(WarpPlaybackAttemptHeaders.HEADER_NAME, TOKEN);
    input.put("x-ghostr-playback-attempt", TOKEN);

    assertThrows(
        IllegalArgumentException.class, () -> WarpPlaybackAttemptHeaders.extract(input));
  }

  @Test
  public void rejectsNonCanonicalTokens() {
    String[] malformed = {"short", "AAAAAAAAAAAAAAAAAAAAA!", "AAAAAAAAAAAAAAAAAAAAAB"};

    for (String token : malformed) {
      assertThrows(
          IllegalArgumentException.class,
          () ->
              WarpPlaybackAttemptHeaders.extract(
                  Map.of(WarpPlaybackAttemptHeaders.HEADER_NAME, token)));
    }
  }
}
