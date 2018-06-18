SELECT
  module_id,
  min(raw_voltage) min_raw_voltage,
  max(raw_voltage) max_raw_voltage
FROM sample
GROUP BY module_id;
