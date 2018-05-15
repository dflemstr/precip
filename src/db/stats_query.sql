SELECT
  sample.module_id,
  slice,
  -- humidity
  min(sample.humidity)                           min_humidity,
  max(sample.humidity)                           max_humidity,
  percentile_cont(0.25)
  WITHIN GROUP (ORDER BY sample.humidity ASC)    p25_humidity,
  percentile_cont(0.50)
  WITHIN GROUP (ORDER BY sample.humidity ASC)    p50_humidity,
  percentile_cont(0.75)
  WITHIN GROUP (ORDER BY sample.humidity ASC)    p75_humidity,
  -- temperature
  min(sample.temperature)                        min_temperature,
  max(sample.temperature)                        max_temperature,
  percentile_cont(0.25)
  WITHIN GROUP (ORDER BY sample.temperature ASC) p25_temperature,
  percentile_cont(0.50)
  WITHIN GROUP (ORDER BY sample.temperature ASC) p50_temperature,
  percentile_cont(0.75)
  WITHIN GROUP (ORDER BY sample.temperature ASC) p75_temperature
FROM generate_series(
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' -
         INTERVAL '23 hours 55 minutes',
         date_trunc('minute', now()) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute',
         '5 minutes') slice
  JOIN sample
    ON date_trunc('minute', sample.created) - (date_part('minute', now()) :: INTEGER % 5) * INTERVAL '1 minute' = slice
GROUP BY sample.module_id, slice
ORDER BY slice DESC, sample.module_id ASC;
