CREATE TABLE global_sample (
  id          SERIAL PRIMARY KEY,
  created     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  temperature DOUBLE PRECISION         NOT NULL
);
