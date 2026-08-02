/^SF:/ {
  file = substr($0, 4)
  ignored = file ~ /^lib\/src\/rust\//
  pure = file ~ /^lib\/core\// || file ~ /\/domain\//
  threshold = pure ? 100 : 95
  hit = 0
  total = 0
  next
}

/^DA:/ && !ignored {
  split(substr($0, 4), data, ",")
  total++
  if (data[2] > 0) {
    hit++
  }
  next
}

/^end_of_record/ && !ignored && total > 0 {
  checked++
  coverage = hit * 100 / total
  if (coverage + 0.000001 < threshold) {
    printf("Dart line coverage %.2f%% is below %.0f%% for %s\n",
           coverage, threshold, file)
    failed = 1
  }
}

END {
  if (checked == 0) {
    print "No Dart coverage records"
    failed = 1
  } else if (!failed) {
    printf("Dart per-file coverage gates passed for %d modules\n", checked)
  }
  exit failed
}
