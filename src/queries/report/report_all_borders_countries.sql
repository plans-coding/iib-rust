WITH RECURSIVE

OrderedEvents AS (
    SELECT InnerId, countriesduringday, Date
    FROM bewb_Events
    WHERE countriesduringday GLOB '[+*a-zA-ZÅÄÖåäö.-]*'
    ORDER BY Date ASC
),

SplittedCountries AS (
    SELECT
        InnerId,
        TRIM(value) AS country,
        Date,
        ROW_NUMBER() OVER (PARTITION BY InnerId ORDER BY Date) AS row_num
    FROM OrderedEvents,
         json_each('["' || REPLACE(countriesduringday, ',', '","') || '"]')
),

ConsecutiveRemoval AS (
    SELECT
        InnerId,
        Date,
        row_num,
        CASE
            WHEN row_num = 1 THEN country
            WHEN country != LAG(country) OVER (PARTITION BY InnerId ORDER BY row_num)
            THEN country
            ELSE NULL
        END AS cleaned_country
    FROM SplittedCountries
),

AllCrossings AS (
    SELECT
        InnerId,
        GROUP_CONCAT(cleaned_country, ' > ') AS AllBorderCrossings
    FROM ConsecutiveRemoval
    WHERE cleaned_country IS NOT NULL
    GROUP BY InnerId
),

-- Split ContinentCountries setting into rows
settings_split(line, rest) AS (
    SELECT
        NULL,
        (SELECT Value FROM bewx_Settings WHERE Attribute = 'ContinentCountries') || char(10)

    UNION ALL

    SELECT
        substr(rest, 1, instr(rest, char(10)) - 1),
        substr(rest, instr(rest, char(10)) + 1)
    FROM settings_split
    WHERE rest <> ''
),

-- Build Country -> ISO lookup
lookup AS (
    SELECT
        TRIM(
            CASE
                WHEN instr(substr(line, instr(line, ':') + 1), ':') > 0 THEN
                    substr(
                        line,
                        instr(line, ':') + 1,
                        instr(substr(line, instr(line, ':') + 1), ':') - 1
                    )
                ELSE
                    substr(line, instr(line, ':') + 1)
            END
        ) AS Country,

        TRIM(
            CASE
                WHEN instr(substr(line, instr(line, ':') + 1), ':') > 0 THEN
                    substr(
                        substr(line, instr(line, ':') + 1),
                        instr(substr(line, instr(line, ':') + 1), ':') + 1
                    )
            END
        ) AS ISO
    FROM settings_split
    WHERE line LIKE '%:%'
),

home_country AS (
    SELECT Value AS Country
    FROM bewx_Settings
    WHERE Attribute = 'HomeCountry'
),

-- First deduplicate countries in travel order
UniqueCountriesBase AS (
    SELECT
        InnerId,
        ltrim(cleaned_country, '*+') AS norm_country,
        MIN(row_num) AS first_pos
    FROM ConsecutiveRemoval
    WHERE cleaned_country IS NOT NULL
      AND cleaned_country NOT LIKE '**%'
      AND ltrim(cleaned_country, '*+') NOT IN (
            SELECT Value
            FROM bewx_Settings
            WHERE Attribute = 'HomeCountry'
      )
    GROUP BY InnerId, norm_country
),

-- Attach flag to each country
UniqueList AS (
    SELECT
        InnerId,
        GROUP_CONCAT(flagged_country, ', ') AS UniqueCountries
    FROM (
        SELECT
            u.InnerId,
            CASE
                WHEN l.ISO IS NOT NULL AND length(l.ISO) = 2 THEN
                    char(
                        127462 + unicode(substr(upper(l.ISO), 1, 1)) - unicode('A'),
                        127462 + unicode(substr(upper(l.ISO), 2, 1)) - unicode('A')
                    ) || ' ' || u.norm_country
                ELSE
                    char(127757) || ' ' || u.norm_country
            END AS flagged_country
        FROM UniqueCountriesBase u
        LEFT JOIN lookup l
               ON l.Country = u.norm_country
        ORDER BY u.InnerId, u.norm_country
    )
    GROUP BY InnerId
),

RouteList AS (
    SELECT
        OuterId,
        GROUP_CONCAT(
            TRIM(
                substr(value, 2, instr(value, ']') - 2)
            ),
            ' > '
        ) AS OverallRoute
    FROM bewa_Overview,
         json_each('["' || replace(MapPins, char(10), '","') || '"]')
    WHERE TRIM(substr(value, 2, instr(value, ']') - 2)) NOT LIKE '@%'
    GROUP BY OuterId
)

SELECT
    b.OuterId,
    a.InnerId,
    a.AllBorderCrossings,
    u.UniqueCountries,
    r.OverallRoute
FROM AllCrossings a
LEFT JOIN UniqueList u USING (InnerId)
LEFT JOIN bewa_Overview b USING (InnerId)
LEFT JOIN RouteList r USING (OuterId)
WHERE b.OuterId = b.OuterId AND a.InnerId = a.InnerId AND b.ParticipantGroup IN (ParticipantGroup) AND b.TripDomain IN (TripDomain) AND 1=1
ORDER BY DepartureDate;
