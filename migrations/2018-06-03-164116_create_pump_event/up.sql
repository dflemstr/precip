CREATE TABLE pump_event (
  id           SERIAL PRIMARY KEY,
  created      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  module_id    INTEGER                  NOT NULL REFERENCES module (id),
  pump_running BOOLEAN                  NOT NULL
);
