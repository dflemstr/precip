SELECT
  sample.module_id,
  slice,
  -- moisture
  min(sample.raw_voltage)                        min_raw_voltage,
  max(sample.raw_voltage)                        max_raw_voltage,
  percentile_cont(0.25)
  WITHIN GROUP (ORDER BY sample.raw_voltage ASC) p25_raw_voltage,
  percentile_cont(0.50)
  WITHIN GROUP (ORDER BY sample.raw_voltage ASC) p50_raw_voltage,
  percentile_cont(0.75)
  WITHIN GROUP (ORDER BY sample.raw_voltage ASC) p75_raw_voltage
FROM generate_series(
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' -
         INTERVAL '71 hours 55 minutes',
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute',
         '5 minutes') slice
  JOIN sample
    ON date_trunc('minute', sample.created) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' = slice
GROUP BY sample.module_id, slice
ORDER BY slice DESC, sample.module_id ASC;
