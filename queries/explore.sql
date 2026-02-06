SELECT
    substr(TripDomain,1,1) ||
    substr(ParticipantGroup,1,1) ||
    printf('%03d',
        ROW_NUMBER() OVER (
        PARTITION BY substr(TripDomain,1,1), substr(ParticipantGroup,1,1)
        ORDER BY DepartureDate
        )
    ) AS OuterId,
    *,
    CAST(julianday(ReturnDate) - julianday(DepartureDate) AS INTEGER) AS NumberOfDays,
    CAST(strftime('%Y', DepartureDate) AS INTEGER) AS TripYear,
    (CAST(strftime('%Y', DepartureDate) AS INTEGER) / 10) * 10 AS TripDecade
FROM
    bewa_Overview
WHERE
    InnerId IS NOT NULL
ORDER BY
    RANDOM()
LIMIT 3;
