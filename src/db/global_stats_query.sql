SELECT last_value(temperature) temperature
FROM global_sample
ORDER BY created ASC;
