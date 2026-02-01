// @generated automatically by Diesel CLI. (Cleaned up manually)

diesel::table! {
    countries (id) {
        id -> Integer,
        name -> Text,
        currency_id -> Nullable<Integer>,
    }
}

diesel::table! {
    currencies (id) {
        id -> Integer,
        name -> Text,
        rate -> Nullable<Double>,
        symbol -> Nullable<Text>,
    }
}

diesel::table! {
    cups (id) {
        id -> Integer,
        name -> Text,
        league_level -> Nullable<Integer>,
        level -> Nullable<Integer>,
        level_index -> Nullable<Integer>,
        match_round -> Nullable<Integer>,
        match_rounds_left -> Nullable<Integer>,
    }
}

diesel::table! {
    languages (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    leagues (id) {
        id -> Integer,
        name -> Text,
        country_id -> Nullable<Integer>,
    }
}

diesel::table! {
    players (id) {
        id -> Integer,
        team_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        player_number -> Integer,
        age -> Integer,
        age_days -> Nullable<Integer>,
        tsi -> Integer,
        player_form -> Integer,
        statement -> Nullable<Text>,
        experience -> Integer,
        loyalty -> Integer,
        mother_club_bonus -> Bool,
        leadership -> Integer,
        salary -> Integer,
        is_abroad -> Bool,
        agreeability -> Integer,
        aggressiveness -> Integer,
        honesty -> Integer,
        league_goals -> Nullable<Integer>,
        cup_goals -> Nullable<Integer>,
        friendlies_goals -> Nullable<Integer>,
        career_goals -> Nullable<Integer>,
        career_hattricks -> Nullable<Integer>,
        speciality -> Nullable<Integer>,
        transfer_listed -> Bool,
        national_team_id -> Nullable<Integer>,
        country_id -> Integer,
        caps -> Nullable<Integer>,
        caps_u20 -> Nullable<Integer>,
        cards -> Nullable<Integer>,
        injury_level -> Nullable<Integer>,
        sticker -> Nullable<Text>,
    }
}

diesel::table! {
    regions (id) {
        id -> Integer,
        name -> Text,
        country_id -> Integer,
    }
}

diesel::table! {
    teams (id) {
        id -> Integer,
        user_id -> Nullable<Integer>,
        name -> Text,
        raw_data -> Text,
        short_name -> Nullable<Text>,
        is_primary_club -> Nullable<Bool>,
        founded_date -> Nullable<Text>,
        arena_id -> Nullable<Integer>,
        arena_name -> Nullable<Text>,
        league_id -> Nullable<Integer>,
        league_name -> Nullable<Text>,
        country_id -> Nullable<Integer>,
        country_name -> Nullable<Text>,
        region_id -> Nullable<Integer>,
        region_name -> Nullable<Text>,
        homepage -> Nullable<Text>,
        dress_uri -> Nullable<Text>,
        dress_alternate_uri -> Nullable<Text>,
        logo_url -> Nullable<Text>,
        trainer_id -> Nullable<Integer>,
        cup_still_in -> Nullable<Bool>,
        cup_id -> Nullable<Integer>,
        cup_name -> Nullable<Text>,
        cup_league_level -> Nullable<Integer>,
        cup_level -> Nullable<Integer>,
        cup_level_index -> Nullable<Integer>,
        cup_match_round -> Nullable<Integer>,
        cup_match_rounds_left -> Nullable<Integer>,
        power_rating_global -> Nullable<Integer>,
        power_rating_league -> Nullable<Integer>,
        power_rating_region -> Nullable<Integer>,
        power_rating_indiv -> Nullable<Integer>,
        friendly_team_id -> Nullable<Integer>,
        league_level_unit_id -> Nullable<Integer>,
        league_level_unit_name -> Nullable<Text>,
        league_level -> Nullable<Integer>,
        number_of_victories -> Nullable<Integer>,
        number_of_undefeated -> Nullable<Integer>,
        number_of_visits -> Nullable<Integer>,
        team_rank -> Nullable<Integer>,
        fanclub_id -> Nullable<Integer>,
        fanclub_name -> Nullable<Text>,
        fanclub_size -> Nullable<Integer>,
        color_background -> Nullable<Text>,
        color_primary -> Nullable<Text>,
        is_bot -> Nullable<Bool>,
        bot_since -> Nullable<Text>,
        youth_team_id -> Nullable<Integer>,
        youth_team_name -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        login_name -> Text,
        supporter_tier -> Text,
        signup_date -> Nullable<Text>,
        activation_date -> Nullable<Text>,
        last_login_date -> Nullable<Text>,
        has_manager_license -> Nullable<Bool>,
        language_id -> Nullable<Integer>,
        language_name -> Nullable<Text>,
    }
}

diesel::joinable!(countries -> currencies (currency_id));
diesel::joinable!(leagues -> countries (country_id));
diesel::joinable!(players -> countries (country_id));
diesel::joinable!(players -> teams (team_id));
diesel::joinable!(regions -> countries (country_id));
diesel::joinable!(teams -> countries (country_id));
diesel::joinable!(teams -> cups (cup_id));
diesel::joinable!(teams -> leagues (league_id));
diesel::joinable!(teams -> regions (region_id));
diesel::joinable!(teams -> users (user_id));
diesel::joinable!(users -> languages (language_id));

diesel::allow_tables_to_appear_in_same_query!(
    countries, currencies, cups, languages, leagues, players, regions, teams, users,
);
