// @generated automatically by Diesel CLI. (Cleaned up manually)

diesel::table! {
    avatars (player_id, download_id) {
        player_id -> Integer,
        download_id -> Integer,
        image -> Binary,
    }
}

diesel::table! {
    countries (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
        currency_id -> Nullable<Integer>,
        country_code -> Nullable<Text>,
        date_format -> Nullable<Text>,
        time_format -> Nullable<Text>,
        flag -> Nullable<Text>,
    }
}

diesel::table! {
    currencies (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
        rate -> Nullable<Double>,
        symbol -> Nullable<Text>,
    }
}

diesel::table! {
    cups (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
        league_level -> Nullable<Integer>,
        level -> Nullable<Integer>,
        level_index -> Nullable<Integer>,
        match_round -> Nullable<Integer>,
        match_rounds_left -> Nullable<Integer>,
    }
}

diesel::table! {
    downloads (id) {
        id -> Integer,
        timestamp -> Text,
        status -> Text,
    }
}

diesel::table! {
    download_entries (id) {
        id -> Integer,
        download_id -> Integer,
        endpoint -> Text,
        version -> Text,
        user_id -> Nullable<Integer>,
        status -> Text,
        fetched_date -> Text,
        error_message -> Nullable<Text>,
        retry_count -> Integer,
    }
}

diesel::table! {
    languages (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    leagues (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
        country_id -> Nullable<Integer>,
        short_name -> Nullable<Text>,
        continent -> Nullable<Text>,
        season -> Nullable<Integer>,
        season_offset -> Nullable<Integer>,
        match_round -> Nullable<Integer>,
        zone_name -> Nullable<Text>,
        english_name -> Nullable<Text>,
        language_id -> Nullable<Integer>,
        national_team_id -> Nullable<Integer>,
        u20_team_id -> Nullable<Integer>,
        active_teams -> Nullable<Integer>,
        active_users -> Nullable<Integer>,
        number_of_levels -> Nullable<Integer>,
        league_system_id -> Integer,
    }
}

diesel::table! {
    players (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        team_id -> Integer,
        first_name -> Text,
        nick_name -> Nullable<Text>,
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
        transfer_listed -> Bool,
        national_team_id -> Nullable<Integer>,
        country_id -> Integer,
        caps -> Nullable<Integer>,
        caps_u20 -> Nullable<Integer>,
        cards -> Nullable<Integer>,
        injury_level -> Nullable<Integer>,
        specialty -> Nullable<Integer>,
        stamina_skill -> Nullable<Integer>,
        keeper_skill -> Nullable<Integer>,
        playmaker_skill -> Nullable<Integer>,
        scorer_skill -> Nullable<Integer>,
        passing_skill -> Nullable<Integer>,
        winger_skill -> Nullable<Integer>,
        defender_skill -> Nullable<Integer>,
        set_pieces_skill -> Nullable<Integer>,
        last_match_date -> Nullable<Text>,
        last_match_id -> Nullable<Integer>,
        last_match_position_code -> Nullable<Integer>,
        last_match_played_minutes -> Nullable<Integer>,
        last_match_rating -> Nullable<Integer>,
        last_match_rating_end_of_match -> Nullable<Integer>,
        arrival_date -> Nullable<Text>,
        player_category_id -> Nullable<Integer>,
        mother_club_team_id -> Nullable<Integer>,
        mother_club_team_name -> Nullable<Text>,
        native_country_id -> Nullable<Integer>,
        native_league_id -> Nullable<Integer>,
        native_league_name -> Nullable<Text>,
        matches_current_team -> Nullable<Integer>,
        goals_current_team -> Nullable<Integer>,
        assists_current_team -> Nullable<Integer>,
        career_assists -> Nullable<Integer>,
        gender_id -> Integer,
    }
}

diesel::table! {
    regions (id, download_id) {
        id -> Integer,
        download_id -> Integer,
        name -> Text,
        country_id -> Integer,
    }
}

diesel::table! {
    teams (id, download_id) {
        id -> Integer,
        download_id -> Integer,
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
        gender_id -> Integer,
    }
}

diesel::table! {
    users (id, download_id) {
        id -> Integer,
        download_id -> Integer,
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

diesel::joinable!(avatars -> downloads (download_id));
diesel::joinable!(countries -> downloads (download_id));
diesel::joinable!(currencies -> downloads (download_id));
diesel::joinable!(download_entries -> downloads (download_id));
diesel::joinable!(leagues -> downloads (download_id));
diesel::joinable!(players -> downloads (download_id));
diesel::joinable!(regions -> downloads (download_id));
diesel::joinable!(teams -> downloads (download_id));
diesel::joinable!(users -> downloads (download_id));

// Retain simple FKs where constraints still essentially exist or for join logic if IDs match
diesel::joinable!(cups -> downloads (download_id));
diesel::joinable!(languages -> downloads (download_id));

diesel::allow_tables_to_appear_in_same_query!(
    avatars,
    countries,
    currencies,
    cups,
    download_entries,
    downloads,
    languages,
    leagues,
    players,
    regions,
    teams,
    users,
);
