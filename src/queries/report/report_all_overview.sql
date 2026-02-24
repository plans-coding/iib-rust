SELECT
    *,
    CAST(strftime('%Y', DepartureDate) AS INTEGER) AS TripYear
FROM bewa_Overview
WHERE
    OuterId IS NOT NULL
    AND TripDomain IN (TripDomain)
    AND ParticipantGroup IN (ParticipantGroup) AND 1=1;
