// @generated automatically by Diesel CLI.

diesel::table! {
    measurements (id) {
        id -> Integer,
        device_id -> Integer,
        timestamp -> BigInt,
        temperature -> Nullable<Float>,
        humidity -> Nullable<Float>,
        pressure -> Nullable<Float>,
        air_quality -> Nullable<Float>,
        v_bat -> Nullable<Float>,
    }
}
