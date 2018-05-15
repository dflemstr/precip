CREATE TABLE module (
  id   SERIAL PRIMARY KEY,
  uuid UUID UNIQUE NOT NULL,
  name TEXT        NOT NULL
);

CREATE TABLE sample (
  id          SERIAL PRIMARY KEY,
  created     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  module_id   INTEGER                  NOT NULL REFERENCES module (id),
  humidity    DOUBLE PRECISION         NOT NULL,
  temperature DOUBLE PRECISION         NOT NULL
);
