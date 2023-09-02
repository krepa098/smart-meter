// @generated automatically by Diesel CLI.

diesel::table! {
    device_names (device_id) {
        device_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    devices (device_id) {
        device_id -> Integer,
        fw_version -> Text,
        bsec_version -> Text,
        wifi_ssid -> Nullable<Text>,
        uptime -> Integer,
        report_interval -> Integer,
        sample_interval -> Integer,
        last_seen -> BigInt,
    }
}

diesel::table! {
    measurements (device_id, timestamp) {
        device_id -> Integer,
        timestamp -> BigInt,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
        pressure -> Nullable<Float>,
        air_quality -> Nullable<Float>,
        bat_v -> Nullable<Float>,
        bat_cap -> Nullable<Float>,
    }
}

diesel::table! {
    measurements_old (id) {
        id -> Integer,
        device_id -> Integer,
        timestamp -> BigInt,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
        pressure -> Nullable<Float>,
        air_quality -> Nullable<Float>,
        bat_v -> Nullable<Float>,
        bat_cap -> Nullable<Float>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    device_names,
    devices,
    measurements,
    measurements_old,
);
