WITH ContinentCountriesParsed AS (
  WITH RECURSIVE split(line, rest) AS (
    SELECT
      substr(Value || char(10), 1, instr(Value || char(10), char(10)) - 1),
      substr(Value || char(10), instr(Value || char(10), char(10)) + 1)
    FROM bewx_Settings
    WHERE Attribute = 'ContinentCountries'
    UNION ALL
    SELECT
      substr(rest, 1, instr(rest, char(10)) - 1),
      substr(rest, instr(rest, char(10)) + 1)
    FROM split
    WHERE rest <> ''
  )
    SELECT
    json_extract(js, '$[0]') AS Continent,
    json_extract(js, '$[1]') AS Country,
    json_extract(js, '$[2]') AS ISO
    FROM (
    SELECT
        '["' || replace(line, ':', '","') || '"]' AS js
    FROM split
    WHERE line <> ''
    )
),
OrderedEvents AS (
    SELECT InnerId, countriesduringday, Date
    FROM bewb_Events
    WHERE countriesduringday GLOB '[+*a-zA-ZÅÄÖåäö.-]*'
    ORDER BY Date ASC
),
SplittedCountries AS (
    SELECT InnerId,
           TRIM(value) AS country,
           Date,
           ROW_NUMBER() OVER (PARTITION BY InnerId ORDER BY Date) AS row_num
    FROM OrderedEvents,
         json_each('["' || REPLACE(countriesduringday, ',', '","') || '"]')
),
ConsecutiveRemoval AS (
    SELECT InnerId, country, Date, row_num,
           CASE
               WHEN row_num = 1 THEN country
               WHEN country != LAG(country) OVER (PARTITION BY InnerId ORDER BY row_num) THEN country
               ELSE NULL
           END AS cleaned_country
    FROM SplittedCountries
),
BorderCrossings AS (
    SELECT b.OuterId, a.InnerId,
           GROUP_CONCAT(a.cleaned_country, ', ') AS AllBorderCrossings
    FROM ConsecutiveRemoval AS a
    LEFT JOIN bewa_Overview AS b ON a.InnerId = b.InnerId
    WHERE a.cleaned_country IS NOT NULL
    GROUP BY a.InnerId
),
normalized AS (
    SELECT
        a.OuterID,
        a.InnerId,
        TRIM(REPLACE(REPLACE(REPLACE(value, '*', ''), '+', ''), '**', '')) AS Country,
        value AS OriginalCountry,
        b.OverallDestination,
        b.ParticipantGroup,
        b.DepartureDate,
        b.TripDomain
    FROM BorderCrossings AS a,
        json_each('["' || REPLACE(AllBorderCrossings, ', ', '", "') || '"]')
    LEFT JOIN  bewa_Overview AS b
    ON b.InnerId = a.InnerId
)
-- Final selection: Removed GROUP_CONCAT and GROUP BY
SELECT
    c.Continent,
    n.Country,
    n.OuterID,
    n.TripDomain,
    n.OverallDestination,
    n.ParticipantGroup
FROM (
    SELECT DISTINCT Country, OuterID, InnerId, TripDomain, OverallDestination, ParticipantGroup
    FROM normalized
    WHERE OriginalCountry NOT LIKE '+%'
    AND OriginalCountry NOT LIKE '**%'
) AS n
LEFT JOIN ContinentCountriesParsed AS c
ON c.Country = n.Country
ORDER BY
    CASE WHEN c.Continent = 'Europa' THEN 0 ELSE 1 END,
    c.Continent ASC,
    n.Country ASC,
    n.OuterID ASC; -- Added OuterID to ordering for cleaner results