WITH Overview AS (
    SELECT
        substr(TripDomain,1,1) ||
        substr(ParticipantGroup,1,1) ||
        printf('%03d',
            ROW_NUMBER() OVER (
                PARTITION BY substr(TripDomain,1,1), substr(ParticipantGroup,1,1)
                ORDER BY DepartureDate
            )
        ) AS OuterId,
        *
    FROM bewa_Overview
)

SELECT
    OuterId,
    *,
    CAST(julianday(ReturnDate) - julianday(DepartureDate) AS INTEGER) AS NumberOfDays,
    TripDomain AS DomainAbbreviation
FROM
    Overview
WHERE
    OuterId = '/*_OUTER_ID_*/'
LIMIT 1;
