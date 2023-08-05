CREATE TABLE trace_files (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp,

    metadata jsonb NOT NULL,

    peak_locations_user int[] CHECK (array_position(peak_locations_user, NULL) IS NULL),
    peak_locations_basecaller int[] CHECK (array_position(peak_locations_basecaller, NULL) IS NULL),

    quality_values_user int[] CHECK (array_position(quality_values_user, NULL) IS NULL),
    quality_values_basecaller int[] CHECK (array_position(quality_values_basecaller, NULL) IS NULL),

    sequences_user int[] CHECK (array_position(sequences_user, NULL) IS NULL),
    sequences_basecaller int[] CHECK (array_position(sequences_basecaller, NULL) IS NULL),

    measurements_voltage int[] CHECK (array_position(measurements_voltage, NULL) IS NULL),
    measurements_current int[] CHECK (array_position(measurements_current, NULL) IS NULL),
    measurements_power int[] CHECK (array_position(measurements_power, NULL) IS NULL),
    measurements_temperature int[] CHECK (array_position(measurements_temperature, NULL) IS NULL),

    analyzed_g int[] CHECK (array_position(analyzed_g, NULL) IS NULL),
    analyzed_a int[] CHECK (array_position(analyzed_a, NULL) IS NULL),
    analyzed_t int[] CHECK (array_position(analyzed_t, NULL) IS NULL),
    analyzed_c int[] CHECK (array_position(analyzed_c, NULL) IS NULL),

    raw_g int[] CHECK (array_position(raw_g, NULL) IS NULL),
    raw_a int[] CHECK (array_position(raw_a, NULL) IS NULL),
    raw_t int[] CHECK (array_position(raw_t, NULL) IS NULL),
    raw_c int[] CHECK (array_position(raw_c, NULL) IS NULL)
);
