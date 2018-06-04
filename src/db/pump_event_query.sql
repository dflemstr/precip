SELECT *
FROM pump_event
WHERE created > now() - INTERVAL '72 hours'
ORDER BY created ASC;
