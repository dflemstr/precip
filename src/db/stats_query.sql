SELECT
  sample.module_id,
  slice,
  -- moisture
  min(sample.moisture)                           min_moisture,
  max(sample.moisture)                           max_moisture,
  percentile_cont(0.25)
  WITHIN GROUP (ORDER BY sample.moisture ASC)    p25_moisture,
  percentile_cont(0.50)
  WITHIN GROUP (ORDER BY sample.moisture ASC)    p50_moisture,
  percentile_cont(0.75)
  WITHIN GROUP (ORDER BY sample.moisture ASC)    p75_moisture
FROM generate_series(
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' -
         INTERVAL '23 hours 55 minutes',
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute',
         '5 minutes') slice
  JOIN sample
    ON date_trunc('minute', sample.created) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' = slice
GROUP BY sample.module_id, slice
ORDER BY slice DESC, sample.module_id ASC;
