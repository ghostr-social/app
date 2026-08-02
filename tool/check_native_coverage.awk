BEGIN {
  threshold["event_identity.rs"] = 100
  threshold["native_models.rs"] = 100
  threshold["event_index.rs"] = 95
  threshold["ffi_models.rs"] = 95
  threshold["gateway_runtime.rs"] = 95
  threshold["http_gateway.rs"] = 95
  threshold["native_cache.rs"] = 95
  threshold["native_gateway.rs"] = 95
  threshold["video_manager.rs"] = 95
}

/^SF:/ {
  path = substr($0, 4)
  parts = split(path, segments, "/")
  file = segments[parts]
  hit = 0
  total = 0
  next
}

/^DA:/ && file in threshold {
  split(substr($0, 4), data, ",")
  total++
  if (data[2] > 0) {
    hit++
  }
  next
}

/^end_of_record/ && file in threshold {
  seen[file] = 1
  coverage = total == 0 ? 0 : hit * 100 / total
  printf("Native line coverage %s: %.2f%% (%d/%d; required %.0f%%)\n",
         file, coverage, hit, total, threshold[file])
  if (coverage + 0.000001 < threshold[file]) {
    failed = 1
  }
}

END {
  for (required in threshold) {
    if (!(required in seen)) {
      printf("Missing native coverage record for %s\n", required)
      failed = 1
    }
  }
  exit failed
}
