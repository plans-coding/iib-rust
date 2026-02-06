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
    TripDomain AS DomainAbbreviation,
    OuterId,
    OverallDestination
FROM
    Overview
WHERE
    OuterId IS NOT NULL
ORDER BY
    DepartureDate ASC;
