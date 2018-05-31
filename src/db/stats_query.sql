SELECT
  module_id,
  min(moisture)        min_moisture,
  max(moisture)        max_moisture,
  last_value(moisture) last_moisture
FROM sample
GROUP BY module_id
ORDER BY created ASC;
