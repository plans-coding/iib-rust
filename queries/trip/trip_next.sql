SELECT
    OuterId,
    TripDomain
FROM bewa_Overview
WHERE OuterId IS NOT NULL
  AND TripDomain IN (TripDomain)
  AND ParticipantGroup IN (ParticipantGroup)
ORDER BY
    DepartureDate <= (
        SELECT DepartureDate
        FROM bewa_Overview
        WHERE OuterId = '_OUTER_ID_'
    ),
    DepartureDate
LIMIT 1;
