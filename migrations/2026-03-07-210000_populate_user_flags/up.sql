-- Retroactively identify the authenticated user for legacy databases.
-- Since SQL cannot call the Hattrick API (e.g. managercompendium or teamdetails),
-- we definitively identify the authenticated user as the one whose teams had their
-- players downloaded. Opponent teams (cached later) never have players downloaded.
UPDATE users 
SET is_current_authenticated_user = 1 
WHERE id IN (
    SELECT DISTINCT user_id 
    FROM teams 
    WHERE id IN (SELECT DISTINCT team_id FROM players)
);
