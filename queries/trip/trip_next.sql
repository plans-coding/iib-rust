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
    TripDomain
FROM Overview
WHERE OuterId IS NOT NULL
  AND DepartureDate > (
    SELECT
		DepartureDate
    FROM Overview
    WHERE OuterId = '_OUTER_ID_'
) AND TripDomain IN (TripDomain) AND ParticipantGroup IN (ParticipantGroup)
ORDER BY DepartureDate ASC
LIMIT 1;
