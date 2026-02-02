ALTER TABLE countries ADD COLUMN country_code TEXT;
ALTER TABLE countries ADD COLUMN date_format TEXT;
ALTER TABLE countries ADD COLUMN time_format TEXT;

ALTER TABLE leagues ADD COLUMN short_name TEXT;
ALTER TABLE leagues ADD COLUMN continent TEXT;
ALTER TABLE leagues ADD COLUMN season INTEGER;
ALTER TABLE leagues ADD COLUMN season_offset INTEGER;
ALTER TABLE leagues ADD COLUMN match_round INTEGER;
ALTER TABLE leagues ADD COLUMN zone_name TEXT;
ALTER TABLE leagues ADD COLUMN english_name TEXT;
ALTER TABLE leagues ADD COLUMN language_id INTEGER;
ALTER TABLE leagues ADD COLUMN national_team_id INTEGER;
ALTER TABLE leagues ADD COLUMN u20_team_id INTEGER;
ALTER TABLE leagues ADD COLUMN active_teams INTEGER;
ALTER TABLE leagues ADD COLUMN active_users INTEGER;
ALTER TABLE leagues ADD COLUMN number_of_levels INTEGER;
