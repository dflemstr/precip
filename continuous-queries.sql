create continuous query range on precip
resample every 1h for 1w
begin
  select
    percentile(moisture, 5) as lo,
    percentile(moisture, 95) as hi
  into
    plant_range
  from
    plant
  group by
    uuid,
    time(1d)
end
