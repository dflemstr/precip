SELECT
  module_id,
  min(moisture)
  OVER w min_moisture,
  max(moisture)
  OVER w max_moisture,
  last_value(moisture)
  OVER w last_moisture
FROM sample
WINDOW w AS (
  PARTITION BY module_id
  ORDER BY created ASC
  RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
);
