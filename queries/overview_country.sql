WITH normalized AS (
                SELECT
                    a.OuterID,
                    a.InnerId,
                    TRIM(REPLACE(REPLACE(REPLACE(value, '*', ''), '+', ''), '**', '')) AS Country,
                    value AS OriginalCountry,
                    b.OverallDestination,
                    b.ParticipantGroup,
                    b.DepartureDate
                FROM IIBc_BorderCrossings AS a,
                    json_each('["' || REPLACE(AllBorderCrossings, ', ', '", "') || '"]')
                LEFT JOIN  bewa_Overview AS b
                ON b.InnerId = a.InnerId
                ORDER BY
                    b.DepartureDate ASC
            )
    SELECT
        c.Continent,
        n.Country,
        GROUP_CONCAT(n.OuterID, ', ') AS OuterIDs,
        GROUP_CONCAT(n.InnerId, ', ') AS InnerIDs,
        GROUP_CONCAT(n.OverallDestination, ' | ') AS OverallDestination,
        GROUP_CONCAT(n.ParticipantGroup, ' | ') AS ParticipantGroup
    FROM (
        SELECT DISTINCT Country, OuterID, InnerId, OverallDestination, ParticipantGroup
        FROM normalized
        WHERE OriginalCountry NOT LIKE '+%'
        AND OriginalCountry NOT LIKE '**%'
    ) AS n
    LEFT JOIN bewx_ContinentCountries AS c
    ON c.Country = n.Country
    GROUP BY c.Continent, n.Country
    ORDER BY
        CASE WHEN c.Continent = 'Europa' THEN 0 ELSE 1 END,
        c.Continent ASC,
        n.Country ASC;
