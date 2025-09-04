CREATE table routes (
    route_id varchar,
    agency_id numeric,
    route_short_name varchar,
    line_id varchar,
    listed_route numeric,
    network_id varchar,
    transit_authority varchar,
    parse_date timestamp
);

CREATE table stop_times (
    trip_id varchar,
    arrival_time timestamp,
    departure_time timestamp,
    stop_id varchar,
    stop_sequence numeric,
    stop_headsign varchar,
    timepoint numeric,
    transit_authority varchar,
    parse_date timestamp
);

CREATE table stops (
    stop_id varchar,
    stop_code varchar,
    stop_name varchar,
    stop_desc varchar,
    zone_id varchar,
    location_type numeric,
    parent_station varchar,
    platform_code varchar,
    transit_authority varchar,
    parse_date timestamp
);

CREATE table trips (
    route_id varchar,
    service_id varchar,
    trip_id varchar,
    trip_headsign varchar,
    trip_short_name varchar,
    direction_id numeric,
    block_id numeric,
    transit_authority varchar,
    parse_date timestamp
);
